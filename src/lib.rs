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
    let module = selectors::register(io::register(error::register(module.name("php_polars"))))
        .class::<dtype::PhpDataType>()
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
        .class::<series::PhpSeries>()
        .class::<frame::PhpDataFrame>()
        .class::<frame::PhpLazyFrame>()
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
