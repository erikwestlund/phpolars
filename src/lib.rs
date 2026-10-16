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
    let module = io::register(error::register(module.name("php_polars")))
        .class::<dtype::PhpDataType>()
        .class::<PhpExpr>()
        .class::<PhpWhen>()
        .class::<PhpThen>()
        .class::<namespace::PhpStrNameSpace>()
        .class::<namespace::PhpDtNameSpace>()
        .class::<namespace::PhpListNameSpace>()
        .class::<namespace::PhpStructNameSpace>()
        .class::<namespace::PhpNameNameSpace>()
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
        .function(wrap_function!(concat))
        .function(wrap_function!(thread_pool_size))
        .function(wrap_function!(version))
        .function(wrap_function!(polars_version))
}
