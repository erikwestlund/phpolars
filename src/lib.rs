#![allow(non_snake_case)]
// PHP methods mirror py-polars keyword parameters, so many take several options.
#![allow(clippy::too_many_arguments)]
use ext_php_rs::{
    flags::IniEntryPermission,
    prelude::*,
    types::{ZendHashTable, Zval},
    zend::{IniEntryDef, IniEntryDefs},
};
use std::{
    ffi::c_char,
    sync::atomic::{AtomicU32, Ordering},
};

mod config;
mod convert;
mod dtype;
mod error;
mod expr;
mod frame;
mod input;
mod io;
mod namespace;
mod selectors;
mod series;
mod sql;

use expr::{PhpExpr, PhpThen, PhpWhen, wrap};

static INI: IniEntryDefs<2> = IniEntryDefs::new([
    // Read once, before the first Polars call in each worker process.
    IniEntryDef::new(c"php_polars.max_threads", c"2", IniEntryPermission::System),
    IniEntryDef::end(),
]);

unsafe extern "C" {
    fn zend_ini_long(name: *const c_char, name_length: usize, orig: i32) -> i64;
}

/// The process that last applied the thread cap. FPM forks workers from the
/// master and (with clear_env) wipes their environment, so each process
/// checks again rather than trusting state copied across fork().
static APPLIED_PID: AtomicU32 = AtomicU32::new(0);

/// Polars sizes its thread pool from POLARS_MAX_THREADS the first time the
/// pool is touched, so this must run in each process before any Polars work:
/// at module startup, at every request startup, and before Polars calls. An
/// explicit environment variable wins over the INI setting.
pub(crate) fn init_threads() {
    let pid = std::process::id();
    if APPLIED_PID.load(Ordering::Relaxed) == pid {
        return;
    }
    if std::env::var_os("POLARS_MAX_THREADS").is_none() {
        let name = c"php_polars.max_threads";
        let n = unsafe { zend_ini_long(name.as_ptr(), name.count_bytes(), 0) }.max(1);
        // SAFETY: this runs once per process, at startup or the first request,
        // before Polars has started any threads, and PHP NTS is single-threaded.
        unsafe { std::env::set_var("POLARS_MAX_THREADS", n.to_string()) };
    }
    APPLIED_PID.store(pid, Ordering::Relaxed);
}

// Expression functions, as py's pl.col(), pl.lit(), pl.when() ...

/// col('a'), col('a', 'b'), col(['a', 'b']) or col('*') for every column.
#[php_function]
#[php(name = "Phpolars\\col")]
pub fn col(names: &[&Zval]) -> PhpResult<PhpExpr> {
    let names = input::names(names)?;
    Ok(wrap(match names.as_slice() {
        [one] if one == "*" => polars::prelude::all().as_expr(),
        [one] => polars::prelude::col(one.as_str()),
        many => polars::prelude::cols(many.to_vec()).as_expr(),
    }))
}

/// A literal; strings stay strings. A PHP list becomes a Series literal.
#[php_function]
#[php(name = "Phpolars\\lit")]
pub fn lit(value: &Zval, dtype: Option<String>) -> PhpResult<PhpExpr> {
    let e = convert::literal(value)?;
    Ok(wrap(match dtype {
        Some(d) => e.strict_cast(dtype::parse(&d)?),
        None => e,
    }))
}

#[php_function]
#[php(name = "Phpolars\\when")]
pub fn when(conditions: &[&Zval]) -> PhpResult<PhpWhen> {
    expr::when_start(conditions)
}

#[php_function]
#[php(name = "Phpolars\\all")]
pub fn all() -> PhpExpr {
    wrap(polars::prelude::all().as_expr())
}

/// Every column except the named ones.
#[php_function]
#[php(name = "Phpolars\\exclude")]
pub fn exclude(names: &[&Zval]) -> PhpResult<PhpExpr> {
    let names = input::names(names)?;
    Ok(wrap(polars::prelude::all().exclude_cols(names).as_expr()))
}

/// The number of rows, as py's pl.len().
#[php_function]
#[php(name = "Phpolars\\len")]
pub fn len() -> PhpExpr {
    wrap(polars::prelude::len())
}

/// The list element inside list()->eval().
#[php_function]
#[php(name = "Phpolars\\element")]
pub fn element() -> PhpExpr {
    wrap(polars::prelude::element())
}

macro_rules! horizontal {
    ($php:literal, $rust:ident, |$es:ident| $body:expr) => {
        #[php_function]
        #[php(name = $php)]
        pub fn $rust(exprs: &[&Zval]) -> PhpResult<PhpExpr> {
            use error::OrPhp;
            let $es = input::exprs(exprs)?;
            $body.php().map(wrap)
        }
    };
}

use polars::lazy::dsl as pl_dsl;

horizontal!("Phpolars\\sum_horizontal", sum_horizontal, |es| {
    pl_dsl::sum_horizontal(es, true)
});
horizontal!("Phpolars\\mean_horizontal", mean_horizontal, |es| {
    pl_dsl::mean_horizontal(es, true)
});
horizontal!("Phpolars\\min_horizontal", min_horizontal, |es| {
    pl_dsl::min_horizontal(es)
});
horizontal!("Phpolars\\max_horizontal", max_horizontal, |es| {
    pl_dsl::max_horizontal(es)
});
horizontal!("Phpolars\\all_horizontal", all_horizontal, |es| {
    pl_dsl::all_horizontal(es)
});
horizontal!("Phpolars\\any_horizontal", any_horizontal, |es| {
    pl_dsl::any_horizontal(es)
});

/// First non-null value across the expressions.
#[php_function]
#[php(name = "Phpolars\\coalesce")]
pub fn coalesce(exprs: &[&Zval]) -> PhpResult<PhpExpr> {
    Ok(wrap(polars::prelude::coalesce(&input::exprs(exprs)?)))
}

#[php_function]
#[php(name = "Phpolars\\concat_str")]
pub fn concat_str(
    exprs: &Zval,
    separator: Option<String>,
    ignoreNulls: Option<bool>,
) -> PhpResult<PhpExpr> {
    let es = input::exprs(&[exprs])?;
    Ok(wrap(polars::prelude::concat_str(
        es,
        &separator.unwrap_or_default(),
        ignoreNulls.unwrap_or(false),
    )))
}

#[php_function]
#[php(name = "Phpolars\\concat_list")]
pub fn concat_list(exprs: &[&Zval]) -> PhpResult<PhpExpr> {
    use error::OrPhp;
    polars::prelude::concat_list(input::exprs(exprs)?)
        .php()
        .map(wrap)
}

/// Pack expressions into one struct column.
#[php_function]
#[php(name = "Phpolars\\struct_")]
pub fn struct_(exprs: &[&Zval]) -> PhpResult<PhpExpr> {
    Ok(wrap(polars::prelude::as_struct(input::exprs(exprs)?)))
}

/// int_range(end) or int_range(start, end, step).
#[php_function]
#[php(name = "Phpolars\\int_range")]
pub fn int_range(start: &Zval, end: Option<&Zval>, step: Option<i64>) -> PhpResult<PhpExpr> {
    let (s, e) = match input::given(end) {
        Some(e) => (input::into_expr(start)?, input::into_expr(e)?),
        None => (polars::prelude::lit(0i64), input::into_expr(start)?),
    };
    Ok(wrap(polars::prelude::int_range(
        s,
        e,
        step.unwrap_or(1),
        polars::prelude::DataType::Int64,
    )))
}

// Aggregation shorthands: sum('a') == col('a')->sum().

macro_rules! agg_fn {
    ($php:literal, $rust:ident, $method:ident) => {
        #[php_function]
        #[php(name = $php)]
        pub fn $rust(names: &[&Zval]) -> PhpResult<PhpExpr> {
            Ok(wrap(col(names)?.inner.$method()))
        }
    };
}

agg_fn!("Phpolars\\sum", sum, sum);
agg_fn!("Phpolars\\mean", mean, mean);
agg_fn!("Phpolars\\median", median, median);
agg_fn!("Phpolars\\min", min, min);
agg_fn!("Phpolars\\max", max, max);
agg_fn!("Phpolars\\first", first, first);
agg_fn!("Phpolars\\last", last, last);
agg_fn!("Phpolars\\n_unique", n_unique, n_unique);

// Ranges, dates, and durations.

fn range_bound(v: Option<&Zval>) -> PhpResult<Option<polars::prelude::Expr>> {
    input::given(v).map(input::operand_or_column).transpose()
}

/// date_range(start, end, interval: '1d', closed: 'both'); start and end are
/// DateTimeInterface values or Expr.
#[php_function]
#[php(name = "Phpolars\\date_range")]
pub fn date_range(
    start: Option<&Zval>,
    end: Option<&Zval>,
    interval: Option<String>,
    numSamples: Option<i64>,
    closed: Option<String>,
) -> PhpResult<PhpExpr> {
    use error::OrPhp;
    let interval = match (&interval, numSamples) {
        (None, Some(_)) => None,
        (i, _) => Some(input::duration(i.as_deref().unwrap_or("1d"))?),
    };
    pl_dsl::date_range(
        range_bound(start)?,
        range_bound(end)?,
        interval,
        numSamples.map(polars::prelude::lit),
        input::closed_window(closed, polars::prelude::ClosedWindow::Both)?,
    )
    .php()
    .map(wrap)
}

#[php_function]
#[php(name = "Phpolars\\date_ranges")]
pub fn date_ranges(
    start: Option<&Zval>,
    end: Option<&Zval>,
    interval: Option<String>,
    closed: Option<String>,
) -> PhpResult<PhpExpr> {
    use error::OrPhp;
    pl_dsl::date_ranges(
        range_bound(start)?,
        range_bound(end)?,
        Some(input::duration(interval.as_deref().unwrap_or("1d"))?),
        None,
        input::closed_window(closed, polars::prelude::ClosedWindow::Both)?,
    )
    .php()
    .map(wrap)
}

#[php_function]
#[php(name = "Phpolars\\datetime_range")]
pub fn datetime_range(
    start: Option<&Zval>,
    end: Option<&Zval>,
    interval: Option<String>,
    numSamples: Option<i64>,
    closed: Option<String>,
    timeUnit: Option<String>,
    timeZone: Option<String>,
) -> PhpResult<PhpExpr> {
    use error::OrPhp;
    let interval = match (&interval, numSamples) {
        (None, Some(_)) => None,
        (i, _) => Some(input::duration(i.as_deref().unwrap_or("1d"))?),
    };
    pl_dsl::datetime_range(
        range_bound(start)?,
        range_bound(end)?,
        interval,
        numSamples.map(polars::prelude::lit),
        input::closed_window(closed, polars::prelude::ClosedWindow::Both)?,
        timeUnit.map(|t| input::time_unit(Some(t))).transpose()?,
        input::time_zone(timeZone)?,
    )
    .php()
    .map(wrap)
}

#[php_function]
#[php(name = "Phpolars\\datetime_ranges")]
pub fn datetime_ranges(
    start: Option<&Zval>,
    end: Option<&Zval>,
    interval: Option<String>,
    closed: Option<String>,
    timeUnit: Option<String>,
    timeZone: Option<String>,
) -> PhpResult<PhpExpr> {
    use error::OrPhp;
    pl_dsl::datetime_ranges(
        range_bound(start)?,
        range_bound(end)?,
        Some(input::duration(interval.as_deref().unwrap_or("1d"))?),
        None,
        input::closed_window(closed, polars::prelude::ClosedWindow::Both)?,
        timeUnit.map(|t| input::time_unit(Some(t))).transpose()?,
        input::time_zone(timeZone)?,
    )
    .php()
    .map(wrap)
}

/// int_ranges(start, end, step): one list of integers per row.
#[php_function]
#[php(name = "Phpolars\\int_ranges")]
pub fn int_ranges(start: &Zval, end: Option<&Zval>, step: Option<&Zval>) -> PhpResult<PhpExpr> {
    let (s, e) = match input::given(end) {
        Some(e) => (input::into_expr(start)?, input::into_expr(e)?),
        None => (polars::prelude::lit(0i64), input::into_expr(start)?),
    };
    let step = match input::given(step) {
        Some(st) => input::into_expr(st)?,
        None => polars::prelude::lit(1i64),
    };
    Ok(wrap(polars::prelude::int_ranges(
        s,
        e,
        step,
        polars::prelude::DataType::Int64,
    )))
}

/// A Date from year, month, and day expressions or numbers.
#[php_function]
#[php(name = "Phpolars\\date")]
pub fn date(year: &Zval, month: &Zval, day: &Zval) -> PhpResult<PhpExpr> {
    let args = polars::prelude::DatetimeArgs::new(
        input::into_expr(year)?,
        input::into_expr(month)?,
        input::into_expr(day)?,
    );
    Ok(wrap(
        polars::prelude::datetime(args).cast(polars::prelude::DataType::Date),
    ))
}

/// A Datetime from its parts; each part is an Expr, a column name, or a number.
#[php_function]
#[php(name = "Phpolars\\datetime")]
pub fn datetime(
    year: &Zval,
    month: &Zval,
    day: &Zval,
    hour: Option<&Zval>,
    minute: Option<&Zval>,
    second: Option<&Zval>,
    microsecond: Option<&Zval>,
    timeUnit: Option<String>,
    timeZone: Option<String>,
) -> PhpResult<PhpExpr> {
    let part = |v: Option<&Zval>| -> PhpResult<polars::prelude::Expr> {
        match input::given(v) {
            Some(v) => input::into_expr(v),
            None => Ok(polars::prelude::lit(0)),
        }
    };
    let args = polars::prelude::DatetimeArgs::new(
        input::into_expr(year)?,
        input::into_expr(month)?,
        input::into_expr(day)?,
    )
    .with_hms(part(hour)?, part(minute)?, part(second)?)
    .with_microsecond(part(microsecond)?)
    .with_time_unit(input::time_unit(timeUnit)?)
    .with_time_zone(input::time_zone(timeZone)?);
    Ok(wrap(polars::prelude::datetime(args)))
}

/// A Duration from parts: duration(days: 1, hours: col('h')).
#[php_function]
#[php(name = "Phpolars\\duration")]
pub fn duration(
    weeks: Option<&Zval>,
    days: Option<&Zval>,
    hours: Option<&Zval>,
    minutes: Option<&Zval>,
    seconds: Option<&Zval>,
    milliseconds: Option<&Zval>,
    microseconds: Option<&Zval>,
    nanoseconds: Option<&Zval>,
    timeUnit: Option<String>,
) -> PhpResult<PhpExpr> {
    let part = |v: Option<&Zval>| -> PhpResult<polars::prelude::Expr> {
        match input::given(v) {
            Some(v) => input::into_expr(v),
            None => Ok(polars::prelude::lit(0)),
        }
    };
    let unit = match timeUnit {
        Some(t) => input::time_unit(Some(t))?,
        None if input::given(nanoseconds).is_some() => polars::prelude::TimeUnit::Nanoseconds,
        None => polars::prelude::TimeUnit::Microseconds,
    };
    let args = polars::prelude::DurationArgs {
        weeks: part(weeks)?,
        days: part(days)?,
        hours: part(hours)?,
        minutes: part(minutes)?,
        seconds: part(seconds)?,
        milliseconds: part(milliseconds)?,
        microseconds: part(microseconds)?,
        nanoseconds: part(nanoseconds)?,
        time_unit: unit,
    };
    Ok(wrap(polars::prelude::duration(args)))
}

/// Integers since the Unix epoch: 'd' gives a Date; 's', 'ms', 'us', and 'ns' give a Datetime.
#[php_function]
#[php(name = "Phpolars\\from_epoch")]
pub fn from_epoch(column: &Zval, timeUnit: Option<String>) -> PhpResult<PhpExpr> {
    use polars::prelude::{DataType, TimeUnit, lit};
    let e = input::into_expr(column)?;
    Ok(wrap(match timeUnit.as_deref().unwrap_or("s") {
        "d" => e.cast(DataType::Date),
        // py scales seconds and milliseconds to microseconds.
        "s" => (e.cast(DataType::Int64) * lit(1_000_000i64))
            .cast(DataType::Datetime(TimeUnit::Microseconds, None)),
        "ms" => (e.cast(DataType::Int64) * lit(1_000i64))
            .cast(DataType::Datetime(TimeUnit::Microseconds, None)),
        u => e.cast(DataType::Datetime(
            input::time_unit(Some(u.to_string()))?,
            None,
        )),
    }))
}

/// Indices where the condition is true.
#[php_function]
#[php(name = "Phpolars\\arg_where")]
pub fn arg_where(condition: &Zval) -> PhpResult<PhpExpr> {
    Ok(wrap(polars::prelude::arg_where(input::into_expr(
        condition,
    )?)))
}

/// Correlation of two columns: method 'pearson' or 'spearman'.
#[php_function]
#[php(name = "Phpolars\\corr")]
pub fn corr(
    a: &Zval,
    b: &Zval,
    method: Option<String>,
    propagateNans: Option<bool>,
) -> PhpResult<PhpExpr> {
    let (a, b) = (input::into_expr(a)?, input::into_expr(b)?);
    Ok(wrap(match method.as_deref().unwrap_or("pearson") {
        "pearson" => polars::prelude::pearson_corr(a, b),
        "spearman" => polars::prelude::spearman_rank_corr(a, b, propagateNans.unwrap_or(false)),
        m => {
            return Err(error::value_error(format!(
                "method must be pearson or spearman, not '{m}'"
            )));
        }
    }))
}

#[php_function]
#[php(name = "Phpolars\\cov")]
pub fn cov(a: &Zval, b: &Zval, ddof: Option<i64>) -> PhpResult<PhpExpr> {
    Ok(wrap(polars::prelude::cov(
        input::into_expr(a)?,
        input::into_expr(b)?,
        input::ddof(ddof)?,
    )))
}

/// Columns by position: nth(0), nth(1, -1).
#[php_function]
#[php(name = "Phpolars\\nth")]
pub fn nth(indices: &[&Zval]) -> PhpResult<PhpExpr> {
    let mut idx = Vec::new();
    for v in indices {
        match (v.long(), v.array()) {
            (Some(i), _) => idx.push(i),
            (None, Some(a)) => {
                for x in a.values() {
                    idx.push(
                        x.long()
                            .ok_or_else(|| error::value_error("nth() takes integer positions"))?,
                    );
                }
            }
            _ => return Err(error::value_error("nth() takes integer positions")),
        }
    }
    let mut it = idx.into_iter();
    let first = it
        .next()
        .ok_or_else(|| error::value_error("nth() needs a position"))?;
    let mut sel = polars::prelude::nth(first);
    for i in it {
        sel |= polars::prelude::nth(i);
    }
    Ok(wrap(sel.as_expr()))
}

/// A value repeated n times.
#[php_function]
#[php(name = "Phpolars\\repeat")]
pub fn repeat(value: &Zval, n: &Zval, dtype: Option<String>) -> PhpResult<PhpExpr> {
    let mut v = input::operand(value)?;
    if let Some(d) = dtype {
        v = v.strict_cast(dtype::parse(&d)?);
    }
    Ok(wrap(polars::prelude::repeat(v, input::into_expr(n)?)))
}

/// num evenly spaced Float64 values from start to end.
#[php_function]
#[php(name = "Phpolars\\linear_space")]
pub fn linear_space(
    start: &Zval,
    end: &Zval,
    numSamples: &Zval,
    closed: Option<String>,
) -> PhpResult<PhpExpr> {
    Ok(wrap(polars::prelude::linear_space(
        input::operand(start)?,
        input::operand(end)?,
        input::into_expr(numSamples)?,
        input::closed_interval(closed)?,
    )))
}

/// Evaluate expressions without a frame: select(lit(1), date_range(...)).
#[php_function]
#[php(name = "Phpolars\\select")]
pub fn select(exprs: &[&Zval]) -> PhpResult<frame::PhpDataFrame> {
    use error::OrPhp;
    init_threads();
    use polars::prelude::IntoLazy;
    polars::prelude::DataFrame::empty()
        .lazy()
        .select(input::exprs(exprs)?)
        .collect()
        .php()
        .map(frame::PhpDataFrame::wrap)
}

/// Each column's values as one list.
#[php_function]
#[php(name = "Phpolars\\implode")]
pub fn implode_fn(names: &[&Zval]) -> PhpResult<PhpExpr> {
    Ok(wrap(col(names)?.inner.implode(true)))
}
agg_fn!(
    "Phpolars\\approx_n_unique",
    approx_n_unique,
    approx_n_unique
);

/// std('a', ddof: 1), var_('a'), quantile('a', 0.5): py's column shorthands.
/// var_ has a trailing underscore because PHP reserves var, as struct_ does for struct.
#[php_function]
#[php(name = "Phpolars\\std")]
pub fn std(column: &Zval, ddof: Option<i64>) -> PhpResult<PhpExpr> {
    Ok(wrap(input::into_expr(column)?.std(input::ddof(ddof)?)))
}

#[php_function]
#[php(name = "Phpolars\\var_")]
pub fn var(column: &Zval, ddof: Option<i64>) -> PhpResult<PhpExpr> {
    Ok(wrap(input::into_expr(column)?.var(input::ddof(ddof)?)))
}

#[php_function]
#[php(name = "Phpolars\\quantile")]
pub fn quantile_fn(
    column: &Zval,
    quantile: input::Num,
    interpolation: Option<String>,
) -> PhpResult<PhpExpr> {
    Ok(wrap(input::into_expr(column)?.quantile(
        polars::prelude::lit(quantile.0),
        expr::quantile_method(interpolation)?,
    )))
}

/// Business days from start (inclusive) to end (exclusive).
#[php_function]
#[php(name = "Phpolars\\business_day_count")]
pub fn business_day_count(
    start: &Zval,
    end: &Zval,
    weekMask: Option<Vec<bool>>,
    holidays: Option<&Zval>,
) -> PhpResult<PhpExpr> {
    let mask: [bool; 7] = match weekMask {
        None => [true, true, true, true, true, false, false],
        Some(m) => m
            .try_into()
            .map_err(|_| error::value_error("weekMask needs seven booleans, Monday to Sunday"))?,
    };
    let date = |v: &Zval| -> PhpResult<polars::prelude::Expr> {
        Ok(input::into_expr(v)?.cast(polars::prelude::DataType::Date))
    };
    Ok(wrap(pl_dsl::business_day_count(
        date(start)?,
        date(end)?,
        mask,
        namespace::holidays_expr(holidays)?,
    )))
}

/// format('{} costs {}', 'item', col('price')): {} placeholders filled in order.
#[php_function]
#[php(name = "Phpolars\\format")]
pub fn format(fString: String, args: &[&Zval]) -> PhpResult<PhpExpr> {
    use error::OrPhp;
    polars::prelude::format_str(&fString, &input::exprs(args)?)
        .php()
        .map(wrap)
}

/// The row order that sorts by the given columns: arg_sort_by(['a', 'b'], descending: [false, true]).
#[php_function]
#[php(name = "Phpolars\\arg_sort_by")]
pub fn arg_sort_by(
    exprs: &Zval,
    descending: Option<&Zval>,
    nullsLast: Option<&Zval>,
    maintainOrder: Option<bool>,
) -> PhpResult<PhpExpr> {
    let options = polars::prelude::SortMultipleOptions::default()
        .with_order_descending_multi(input::bools(descending, false)?)
        .with_nulls_last_multi(input::bools(nullsLast, false)?)
        .with_maintain_order(maintainOrder.unwrap_or(false));
    Ok(wrap(polars::prelude::arg_sort_by(
        input::exprs(&[exprs])?,
        options,
    )))
}

/// Combine columns into one fixed-width Array column.
#[php_function]
#[php(name = "Phpolars\\concat_arr")]
pub fn concat_arr(exprs: &[&Zval]) -> PhpResult<PhpExpr> {
    use error::OrPhp;
    polars::prelude::concat_arr(input::exprs(exprs)?)
        .php()
        .map(wrap)
}

/// A struct field inside struct()->withFields(): field('a').
#[php_function]
#[php(name = "Phpolars\\field")]
pub fn field(name: &Zval) -> PhpResult<PhpExpr> {
    let names: Vec<polars::prelude::PlSmallStr> =
        input::names(&[name])?.into_iter().map(Into::into).collect();
    Ok(wrap(polars::prelude::Expr::Field(names.into())))
}

/// Collect several LazyFrames at once, sharing work between their plans.
#[php_function]
#[php(name = "Phpolars\\collect_all")]
pub fn collect_all(lazyFrames: &ZendHashTable) -> PhpResult<Vec<frame::PhpDataFrame>> {
    use error::OrPhp;
    let mut plans = Vec::new();
    for v in lazyFrames.values() {
        let lf = v
            .extract::<&frame::PhpLazyFrame>()
            .ok_or_else(|| error::value_error("collect_all() takes a list of LazyFrames"))?;
        plans.push(lf.inner.clone().logical_plan);
    }
    init_threads();
    polars::prelude::LazyFrame::collect_all_with_engine(
        plans,
        polars::prelude::Engine::Auto,
        polars::prelude::OptFlags::default(),
    )
    .php()
    .map(|dfs| dfs.into_iter().map(frame::PhpDataFrame::wrap).collect())
}

/// py's from_dict, from_dicts, and from_records; the same as the DataFrame constructors.
#[php_function]
#[php(name = "Phpolars\\from_dict")]
pub fn from_dict(
    data: &ZendHashTable,
    schema: Option<&ZendHashTable>,
) -> PhpResult<frame::PhpDataFrame> {
    convert::from_columns(data, schema).map(frame::PhpDataFrame::wrap)
}

#[php_function]
#[php(name = "Phpolars\\from_dicts")]
pub fn from_dicts(
    data: &ZendHashTable,
    schema: Option<&ZendHashTable>,
) -> PhpResult<frame::PhpDataFrame> {
    convert::from_rows(data, schema).map(frame::PhpDataFrame::wrap)
}

#[php_function]
#[php(name = "Phpolars\\from_records")]
pub fn from_records(
    data: &ZendHashTable,
    schema: Option<&ZendHashTable>,
) -> PhpResult<frame::PhpDataFrame> {
    convert::from_rows(data, schema).map(frame::PhpDataFrame::wrap)
}

/// Run a query on a PDO connection and read the result: read_database('SELECT ...', $pdo).
/// params binds placeholders through prepare() and execute(); schemaOverrides casts
/// columns afterwards, for drivers that return numbers as strings.
#[php_function]
#[php(name = "Phpolars\\read_database")]
pub fn read_database(
    query: String,
    connection: &Zval,
    params: Option<&ZendHashTable>,
    schemaOverrides: Option<&ZendHashTable>,
) -> PhpResult<frame::PhpDataFrame> {
    use error::OrPhp;
    use ext_php_rs::convert::IntoZval;
    let statement = match params {
        Some(p) => {
            let stmt = connection.try_call_method("prepare", vec![&query])?;
            stmt.try_call_method("execute", vec![&p.to_owned().into_zval(false)?])?;
            stmt
        }
        None => connection.try_call_method("query", vec![&query])?,
    };
    if statement.is_false() {
        return Err(error::value_error(
            "the query failed; enable PDO::ERRMODE_EXCEPTION for the driver's message",
        ));
    }
    // PDO::FETCH_ASSOC
    let rows = statement.try_call_method("fetchAll", vec![&2i64])?;
    let rows = rows
        .array()
        .ok_or_else(|| error::value_error("fetchAll() did not return an array"))?;
    let mut df = if rows.is_empty() {
        // No rows: take the column names from the statement's metadata.
        let n = statement
            .try_call_method("columnCount", vec![])?
            .long()
            .unwrap_or(0);
        let mut columns = Vec::new();
        for i in 0..n {
            let meta = statement.try_call_method("getColumnMeta", vec![&i])?;
            let name = meta
                .array()
                .and_then(|m| m.get("name"))
                .and_then(|v| v.str())
                .unwrap_or("")
                .to_string();
            columns.push(polars::prelude::Column::full_null(
                name.into(),
                0,
                &polars::prelude::DataType::Null,
            ));
        }
        polars::prelude::DataFrame::new_infer_height(columns).php()?
    } else {
        convert::from_rows(rows, None)?
    };
    if let Some(o) = schemaOverrides {
        use polars::prelude::{DataType, IntoLazy, StrptimeOptions, col, lit};
        let mut exprs = Vec::new();
        for (name, target) in convert::schema_overrides(Some(o))? {
            let source = df
                .column(&name)
                .map_err(|_| error::column_not_found(&name))?
                .dtype()
                .clone();
            let c = col(name.as_str());
            // Text from the driver is parsed, since Polars deprecates casting it to temporal types.
            exprs.push(match (&source, &target) {
                (DataType::String, DataType::Date) => c.str().to_date(StrptimeOptions::default()),
                (DataType::String, DataType::Datetime(tu, tz)) => c.str().to_datetime(
                    Some(*tu),
                    tz.clone(),
                    StrptimeOptions::default(),
                    lit("raise"),
                ),
                (DataType::String, DataType::Time) => c.str().to_time(StrptimeOptions::default()),
                _ => c.strict_cast(target),
            });
        }
        init_threads();
        df = df.lazy().with_columns(exprs).collect().php()?;
    }
    Ok(frame::PhpDataFrame::wrap(df))
}

/// any('a') == col('a')->any(): true if any value is true.
#[php_function]
#[php(name = "Phpolars\\any")]
pub fn any(names: &Zval, ignoreNulls: Option<bool>) -> PhpResult<PhpExpr> {
    Ok(wrap(col(&[names])?.inner.any(ignoreNulls.unwrap_or(true))))
}

/// The angle of (x, y) in radians: arctan2(col('y'), col('x')).
#[php_function]
#[php(name = "Phpolars\\arctan2")]
pub fn arctan2(y: &Zval, x: &Zval) -> PhpResult<PhpExpr> {
    Ok(wrap(input::into_expr(y)?.arctan2(input::into_expr(x)?)))
}

#[php_function]
#[php(name = "Phpolars\\cum_sum")]
pub fn cum_sum(names: &Zval, reverse: Option<bool>) -> PhpResult<PhpExpr> {
    Ok(wrap(col(&[names])?.inner.cum_sum(reverse.unwrap_or(false))))
}

/// Running count of rows, or of non-null values of the given columns.
#[php_function]
#[php(name = "Phpolars\\cum_count")]
pub fn cum_count(names: &Zval, reverse: Option<bool>) -> PhpResult<PhpExpr> {
    Ok(wrap(
        col(&[names])?.inner.cum_count(reverse.unwrap_or(false)),
    ))
}

/// Running sums across columns, as a struct named cum_sum.
#[php_function]
#[php(name = "Phpolars\\cum_sum_horizontal")]
pub fn cum_sum_horizontal(exprs: &[&Zval]) -> PhpResult<PhpExpr> {
    let es = input::exprs(exprs)?;
    let mut acc: Option<polars::prelude::Expr> = None;
    let mut fields = Vec::with_capacity(es.len());
    for e in es {
        let name = e
            .clone()
            .meta()
            .output_name()
            .map_err(error::polars_error)?;
        let next = match acc {
            None => e,
            Some(a) => a + e,
        };
        fields.push(next.clone().alias(name));
        acc = Some(next);
    }
    Ok(wrap(polars::prelude::as_struct(fields).alias("cum_sum")))
}

/// The string with regex metacharacters escaped.
#[php_function]
#[php(name = "Phpolars\\escape_regex")]
pub fn escape_regex(s: String) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if "\\.+*?()|[]{}^$#&-~".contains(c) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// head('a', 10) == col('a')->head(10).
#[php_function]
#[php(name = "Phpolars\\head")]
pub fn head(names: &Zval, n: Option<i64>) -> PhpResult<PhpExpr> {
    Ok(wrap(
        col(&[names])?
            .inner
            .head(Some(input::count(n.unwrap_or(10), "n")?)),
    ))
}

#[php_function]
#[php(name = "Phpolars\\tail")]
pub fn tail(names: &Zval, n: Option<i64>) -> PhpResult<PhpExpr> {
    Ok(wrap(
        col(&[names])?
            .inner
            .tail(Some(input::count(n.unwrap_or(10), "n")?)),
    ))
}

/// One list (or Array, with asArray) of evenly spaced values per row.
#[php_function]
#[php(name = "Phpolars\\linear_spaces")]
pub fn linear_spaces(
    start: &Zval,
    end: &Zval,
    numSamples: &Zval,
    closed: Option<String>,
    asArray: Option<bool>,
) -> PhpResult<PhpExpr> {
    use error::OrPhp;
    polars::prelude::linear_spaces(
        input::into_expr(start)?,
        input::into_expr(end)?,
        input::into_expr(numSamples)?,
        input::closed_interval(closed)?,
        asArray.unwrap_or(false),
    )
    .php()
    .map(wrap)
}

/// n ones or zeros of a type (default Float64).
#[php_function]
#[php(name = "Phpolars\\ones")]
pub fn ones(n: &Zval, dtype: Option<String>) -> PhpResult<PhpExpr> {
    let d = dtype::parse(dtype.as_deref().unwrap_or("Float64"))?;
    Ok(wrap(polars::prelude::repeat(
        polars::prelude::lit(1).cast(d),
        input::into_expr(n)?,
    )))
}

#[php_function]
#[php(name = "Phpolars\\zeros")]
pub fn zeros(n: &Zval, dtype: Option<String>) -> PhpResult<PhpExpr> {
    let d = dtype::parse(dtype.as_deref().unwrap_or("Float64"))?;
    Ok(wrap(polars::prelude::repeat(
        polars::prelude::lit(0).cast(d),
        input::into_expr(n)?,
    )))
}

/// The row number, 0, 1, 2, ..., as UInt32.
#[php_function]
#[php(name = "Phpolars\\row_index")]
pub fn row_index(name: Option<String>) -> PhpExpr {
    wrap(
        polars::prelude::int_range(
            polars::prelude::lit(0u32),
            polars::prelude::len(),
            1,
            polars::prelude::DataType::UInt32,
        )
        .alias(name.unwrap_or_else(|| "index".into())),
    )
}

/// Seed Polars' random number generator, for sample() and shuffle().
#[php_function]
#[php(name = "Phpolars\\set_random_seed")]
pub fn set_random_seed(seed: i64) {
    polars_core::random::set_global_random_seed(seed as u64);
}

/// py's threadpool_size(); the same as thread_pool_size().
#[php_function]
#[php(name = "Phpolars\\threadpool_size")]
pub fn threadpool_size() -> i64 {
    thread_pool_size()
}

/// A Time from parts: time(12, 30), time(col('h'), col('m')).
#[php_function]
#[php(name = "Phpolars\\time")]
pub fn time(
    hour: Option<&Zval>,
    minute: Option<&Zval>,
    second: Option<&Zval>,
    microsecond: Option<&Zval>,
) -> PhpResult<PhpExpr> {
    let part = |v: Option<&Zval>| -> PhpResult<polars::prelude::Expr> {
        match input::given(v) {
            Some(v) => input::into_expr(v),
            None => Ok(polars::prelude::lit(0)),
        }
    };
    let args = polars::prelude::DatetimeArgs::new(
        polars::prelude::lit(1970),
        polars::prelude::lit(1),
        polars::prelude::lit(1),
    )
    .with_hms(part(hour)?, part(minute)?, part(second)?)
    .with_microsecond(part(microsecond)?);
    Ok(wrap(
        polars::prelude::datetime(args).cast(polars::prelude::DataType::Time),
    ))
}

fn time_bound(v: Option<&Zval>, default: &str) -> PhpResult<polars::prelude::Expr> {
    use polars::prelude::{StrptimeOptions, lit};
    match input::given(v) {
        Some(v) if v.str().is_some() => Ok(lit(v.str().unwrap())
            .str()
            .to_time(StrptimeOptions::default())),
        Some(v) => input::into_expr(v),
        None => Ok(lit(default).str().to_time(StrptimeOptions::default())),
    }
}

/// Times from start to end ('HH:MM:SS' text or Expr) every interval.
#[php_function]
#[php(name = "Phpolars\\time_range")]
pub fn time_range(
    start: Option<&Zval>,
    end: Option<&Zval>,
    interval: Option<String>,
    closed: Option<String>,
) -> PhpResult<PhpExpr> {
    Ok(wrap(pl_dsl::time_range(
        time_bound(start, "00:00:00")?,
        time_bound(end, "23:59:59.999999")?,
        input::duration(interval.as_deref().unwrap_or("1h"))?,
        input::closed_window(closed, polars::prelude::ClosedWindow::Both)?,
    )))
}

#[php_function]
#[php(name = "Phpolars\\time_ranges")]
pub fn time_ranges(
    start: Option<&Zval>,
    end: Option<&Zval>,
    interval: Option<String>,
    closed: Option<String>,
) -> PhpResult<PhpExpr> {
    Ok(wrap(pl_dsl::time_ranges(
        time_bound(start, "00:00:00")?,
        time_bound(end, "23:59:59.999999")?,
        input::duration(interval.as_deref().unwrap_or("1h"))?,
        input::closed_window(closed, polars::prelude::ClosedWindow::Both)?,
    )))
}

fn rolling_cov_options(
    windowSize: i64,
    minPeriods: Option<i64>,
    ddof: Option<i64>,
) -> PhpResult<polars::prelude::RollingCovOptions> {
    let w = input::count(windowSize, "windowSize")?;
    Ok(polars::prelude::RollingCovOptions {
        window_size: w as polars::prelude::IdxSize,
        min_periods: input::count(minPeriods.unwrap_or(windowSize), "minPeriods")?
            as polars::prelude::IdxSize,
        ddof: input::ddof(ddof)?,
    })
}

#[php_function]
#[php(name = "Phpolars\\rolling_corr")]
pub fn rolling_corr(
    a: &Zval,
    b: &Zval,
    windowSize: i64,
    minPeriods: Option<i64>,
    ddof: Option<i64>,
) -> PhpResult<PhpExpr> {
    Ok(wrap(polars::prelude::rolling_corr(
        input::into_expr(a)?,
        input::into_expr(b)?,
        rolling_cov_options(windowSize, minPeriods, ddof)?,
    )))
}

#[php_function]
#[php(name = "Phpolars\\rolling_cov")]
pub fn rolling_cov(
    a: &Zval,
    b: &Zval,
    windowSize: i64,
    minPeriods: Option<i64>,
    ddof: Option<i64>,
) -> PhpResult<PhpExpr> {
    Ok(wrap(polars::prelude::rolling_cov(
        input::into_expr(a)?,
        input::into_expr(b)?,
        rolling_cov_options(windowSize, minPeriods, ddof)?,
    )))
}

/// py's union(): concat() without a promise about row order.
#[php_function]
#[php(name = "Phpolars\\union")]
pub fn union(items: &ZendHashTable, how: Option<String>) -> PhpResult<Zval> {
    frame::concat(items, how)
}

/// py's arange(); the same as int_range().
#[php_function]
#[php(name = "Phpolars\\arange")]
pub fn arange(start: &Zval, end: Option<&Zval>, step: Option<i64>) -> PhpResult<PhpExpr> {
    int_range(start, end, step)
}

/// An expression from SQL text: sql_expr('a * 2 AS b').
#[php_function]
#[php(name = "Phpolars\\sql_expr")]
pub fn sql_expr(sql: String) -> PhpResult<PhpExpr> {
    #[cfg(feature = "sql")]
    {
        use error::OrPhp;
        polars::sql::sql_expr(&sql).php().map(wrap)
    }
    #[cfg(not(feature = "sql"))]
    {
        let _ = sql;
        Err(PhpException::from_class::<error::ComputeError>(
            "php_polars was built without the 'sql' feature".to_string(),
        ))
    }
}

/// Flatten nested associative arrays into columns named with separator:
/// json_normalize([['a' => 1, 'b' => ['c' => 2]]]) has columns a and b.c.
/// Values nested deeper than maxLevel stay as JSON text.
#[php_function]
#[php(name = "Phpolars\\json_normalize")]
pub fn json_normalize(
    data: &ZendHashTable,
    separator: Option<String>,
    maxLevel: Option<i64>,
) -> PhpResult<frame::PhpDataFrame> {
    let sep = separator.unwrap_or_else(|| ".".into());
    let records: Vec<&ZendHashTable> = if convert::is_list(data) {
        data.values()
            .map(|v| {
                v.array()
                    .ok_or_else(|| error::value_error("json_normalize() takes a list of arrays"))
            })
            .collect::<PhpResult<_>>()?
    } else {
        vec![data]
    };
    let mut order: Vec<String> = Vec::new();
    let mut flat: Vec<Vec<(String, Zval)>> = Vec::with_capacity(records.len());
    for r in records {
        let mut out = Vec::new();
        // Top-level plain values come first, then the flattened nested ones, as in py.
        for (k, v) in r.iter() {
            if !is_dict(v) {
                out.push((k.to_string(), v.shallow_clone()));
            }
        }
        for (k, v) in r.iter() {
            if is_dict(v) {
                normalize_into(&k.to_string(), v, &sep, maxLevel, 1, &mut out)?;
            }
        }
        for (k, _) in &out {
            if !order.contains(k) {
                order.push(k.clone());
            }
        }
        flat.push(out);
    }
    let mut columns = ext_php_rs::types::ZendHashTable::new();
    for name in &order {
        let mut values = ext_php_rs::types::ZendHashTable::with_capacity(flat.len() as u32);
        for row in &flat {
            match row.iter().find(|(k, _)| k == name) {
                Some((_, v)) => values.push(v.shallow_clone())?,
                None => values.push(())?,
            }
        }
        columns.insert(name.as_str(), values)?;
    }
    convert::from_columns(&columns, None).map(frame::PhpDataFrame::wrap)
}

fn is_dict(v: &Zval) -> bool {
    v.array()
        .is_some_and(|a| !a.is_empty() && !convert::is_list(a))
}

fn normalize_into(
    prefix: &str,
    v: &Zval,
    sep: &str,
    max_level: Option<i64>,
    level: i64,
    out: &mut Vec<(String, Zval)>,
) -> PhpResult<()> {
    for (k, child) in v.array().unwrap().iter() {
        let key = format!("{prefix}{sep}{k}");
        if is_dict(child) && max_level.is_none_or(|m| level < m) {
            normalize_into(&key, child, sep, max_level, level + 1, out)?;
        } else if is_dict(child) {
            use ext_php_rs::convert::IntoZval;
            out.push((key, py_json(child)?.into_zval(false)?));
        } else {
            out.push((key, child.shallow_clone()));
        }
    }
    Ok(())
}

/// JSON text spaced as Python's json.dumps writes it, so results match py.
fn py_json(v: &Zval) -> PhpResult<String> {
    if let Some(a) = v.array() {
        if convert::is_list(a) {
            let items: Vec<String> = a.values().map(py_json).collect::<PhpResult<_>>()?;
            return Ok(format!("[{}]", items.join(", ")));
        }
        let mut items = Vec::new();
        for (k, x) in a.iter() {
            items.push(format!(
                "{}: {}",
                py_json(&{
                    use ext_php_rs::convert::IntoZval;
                    k.to_string().into_zval(false)?
                })?,
                py_json(x)?
            ));
        }
        return Ok(format!("{{{}}}", items.join(", ")));
    }
    // JSON_UNESCAPED_SLASHES | JSON_PRESERVE_ZERO_FRACTION
    let enc = ext_php_rs::types::ZendCallable::try_from_name("json_encode")?;
    Ok(enc
        .try_call(vec![v, &(64i64 | 1024)])?
        .str()
        .unwrap_or("null")
        .to_string())
}

/// The column names and types of a Parquet file, without reading its data.
#[php_function]
#[php(name = "Phpolars\\read_parquet_schema")]
pub fn read_parquet_schema(source: String) -> PhpResult<Vec<(String, String)>> {
    use error::OrPhp;
    let lf = io::scan_parquet(source, None, None, None, None, None, None, None, None, None)?;
    Ok(dtype::schema_array(
        lf.inner.clone().collect_schema().php()?.as_ref(),
    ))
}

#[php_function]
#[php(name = "Phpolars\\read_ipc_schema")]
pub fn read_ipc_schema(source: String) -> PhpResult<Vec<(String, String)>> {
    use error::OrPhp;
    let lf = io::scan_ipc(source, None, None, None)?;
    Ok(dtype::schema_array(
        lf.inner.clone().collect_schema().php()?.as_ref(),
    ))
}

/// count('a') counts non-null values; count() alone counts rows.
#[php_function]
#[php(name = "Phpolars\\count")]
pub fn count(names: &[&Zval]) -> PhpResult<PhpExpr> {
    if names.is_empty() {
        return Ok(len());
    }
    Ok(wrap(col(names)?.inner.count()))
}

/// concat([$a, $b], how: 'vertical'|'vertical_relaxed'|'horizontal'|'diagonal'|'diagonal_relaxed')
#[php_function]
#[php(name = "Phpolars\\concat")]
pub fn concat(items: &ZendHashTable, how: Option<String>) -> PhpResult<Zval> {
    frame::concat(items, how)
}

/// The size of the Polars thread pool in this process.
#[php_function]
#[php(name = "Phpolars\\thread_pool_size")]
pub fn thread_pool_size() -> i64 {
    init_threads();
    polars_core::runtime::THREAD_POOL.current_num_threads() as i64
}

#[php_function]
#[php(name = "Phpolars\\version")]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// The Polars crate version the extension is built on. Kept in step with
/// the pin in Cargo.toml; tests/dataframe.php checks that they agree.
#[php_function]
#[php(name = "Phpolars\\polars_version")]
pub fn polars_version() -> &'static str {
    "0.55.2"
}

fn startup(_ty: i32, mod_num: i32) -> i32 {
    IniEntryDef::register(INI.as_slice(), mod_num);
    // Apply the cap before any PHP code runs: eager operations can start
    // Polars' pool without passing through init_threads().
    init_threads();
    0
}

/// RINIT: forked FPM workers apply the cap again, since their environment
/// may have been cleared after the master's startup.
unsafe extern "C" fn request_startup(_ty: i32, _module_number: i32) -> i32 {
    init_threads();
    0
}

#[php_module]
#[php(startup = startup)]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    let module = sql::register(selectors::register(io::register(error::register(
        module.name("php_polars"),
    ))))
    .class::<dtype::PhpDataType>()
    .class::<config::PhpConfig>()
    .class::<PhpExpr>()
    .class::<PhpWhen>()
    .class::<PhpThen>()
    .class::<namespace::PhpStrNameSpace>()
    .class::<namespace::PhpDtNameSpace>()
    .class::<namespace::PhpListNameSpace>()
    .class::<namespace::PhpStructNameSpace>()
    .class::<namespace::PhpNameNameSpace>()
    .class::<namespace::PhpCatNameSpace>()
    .class::<namespace::PhpArrNameSpace>()
    .class::<namespace::PhpBinNameSpace>()
    .class::<namespace::PhpMetaNameSpace>()
    .class::<series::PhpSeries>()
    .class::<frame::PhpDataFrame>()
    .class::<frame::PhpLazyFrame>()
    .class::<frame::GroupIterator>()
    .class::<frame::PhpGroupBy>()
    .class::<frame::PhpLazyGroupBy>();
    module
        .request_startup_function(request_startup)
        .function(wrap_function!(col))
        .function(wrap_function!(lit))
        .function(wrap_function!(when))
        .function(wrap_function!(all))
        .function(wrap_function!(exclude))
        .function(wrap_function!(len))
        .function(wrap_function!(element))
        .function(wrap_function!(sum_horizontal))
        .function(wrap_function!(mean_horizontal))
        .function(wrap_function!(min_horizontal))
        .function(wrap_function!(max_horizontal))
        .function(wrap_function!(all_horizontal))
        .function(wrap_function!(any_horizontal))
        .function(wrap_function!(coalesce))
        .function(wrap_function!(concat_str))
        .function(wrap_function!(concat_list))
        .function(wrap_function!(struct_))
        .function(wrap_function!(int_range))
        .function(wrap_function!(sum))
        .function(wrap_function!(mean))
        .function(wrap_function!(median))
        .function(wrap_function!(min))
        .function(wrap_function!(max))
        .function(wrap_function!(first))
        .function(wrap_function!(last))
        .function(wrap_function!(n_unique))
        .function(wrap_function!(count))
        .function(wrap_function!(business_day_count))
        .function(wrap_function!(format))
        .function(wrap_function!(arg_sort_by))
        .function(wrap_function!(concat_arr))
        .function(wrap_function!(field))
        .function(wrap_function!(collect_all))
        .function(wrap_function!(from_dict))
        .function(wrap_function!(from_dicts))
        .function(wrap_function!(from_records))
        .function(wrap_function!(read_database))
        .function(wrap_function!(any))
        .function(wrap_function!(arctan2))
        .function(wrap_function!(cum_sum))
        .function(wrap_function!(cum_count))
        .function(wrap_function!(cum_sum_horizontal))
        .function(wrap_function!(escape_regex))
        .function(wrap_function!(head))
        .function(wrap_function!(tail))
        .function(wrap_function!(linear_spaces))
        .function(wrap_function!(ones))
        .function(wrap_function!(zeros))
        .function(wrap_function!(row_index))
        .function(wrap_function!(set_random_seed))
        .function(wrap_function!(threadpool_size))
        .function(wrap_function!(time))
        .function(wrap_function!(time_range))
        .function(wrap_function!(time_ranges))
        .function(wrap_function!(rolling_corr))
        .function(wrap_function!(rolling_cov))
        .function(wrap_function!(union))
        .function(wrap_function!(arange))
        .function(wrap_function!(sql_expr))
        .function(wrap_function!(json_normalize))
        .function(wrap_function!(read_parquet_schema))
        .function(wrap_function!(read_ipc_schema))
        .function(wrap_function!(date_range))
        .function(wrap_function!(date_ranges))
        .function(wrap_function!(datetime_range))
        .function(wrap_function!(datetime_ranges))
        .function(wrap_function!(int_ranges))
        .function(wrap_function!(date))
        .function(wrap_function!(datetime))
        .function(wrap_function!(duration))
        .function(wrap_function!(from_epoch))
        .function(wrap_function!(arg_where))
        .function(wrap_function!(corr))
        .function(wrap_function!(cov))
        .function(wrap_function!(nth))
        .function(wrap_function!(repeat))
        .function(wrap_function!(linear_space))
        .function(wrap_function!(select))
        .function(wrap_function!(implode_fn))
        .function(wrap_function!(approx_n_unique))
        .function(wrap_function!(std))
        .function(wrap_function!(var))
        .function(wrap_function!(quantile_fn))
        .function(wrap_function!(concat))
        .function(wrap_function!(thread_pool_size))
        .function(wrap_function!(version))
        .function(wrap_function!(polars_version))
}
