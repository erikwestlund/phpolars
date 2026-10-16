use ext_php_rs::{
    flags::IniEntryPermission,
    prelude::*,
    zend::{IniEntryDef, IniEntryDefs},
};
use std::{ffi::c_char, sync::Once};

mod convert;
mod expr;
mod frame;

pub use expr::PhpExpr;
pub use frame::{PhpDataFrame, PhpGroupBy};

static INI: IniEntryDefs<2> = IniEntryDefs::new([
    // Read once, before the first Polars call in each worker process.
    IniEntryDef::new(c"php_polars.max_threads", c"2", IniEntryPermission::System),
    IniEntryDef::end(),
]);

unsafe extern "C" {
    fn zend_ini_long(name: *const c_char, name_length: usize, orig: i32) -> i64;
}

static THREADS: Once = Once::new();

/// Polars sizes its thread pool from POLARS_MAX_THREADS the first time the
/// pool is touched, so this must run before any Polars work. An explicit
/// environment variable wins over the INI setting.
pub(crate) fn init_threads() {
    THREADS.call_once(|| {
        if std::env::var_os("POLARS_MAX_THREADS").is_some() {
            return;
        }
        let name = c"php_polars.max_threads";
        let n = unsafe { zend_ini_long(name.as_ptr(), name.count_bytes(), 0) }.max(1);
        // SAFETY: PHP NTS runs one thread per process and Polars has not
        // started its pool yet, so nothing else reads the environment.
        unsafe { std::env::set_var("POLARS_MAX_THREADS", n.to_string()) };
    });
}

pub(crate) fn polars_error(e: polars::prelude::PolarsError) -> PhpException {
    PhpException::from(e.to_string())
}

#[php_function]
#[php(name = "Phpolars\\col")]
pub fn col(name: String) -> PhpExpr {
    PhpExpr::col(name)
}

#[php_function]
#[php(name = "Phpolars\\lit")]
pub fn lit(value: &ext_php_rs::types::Zval) -> PhpResult<PhpExpr> {
    PhpExpr::lit(value)
}

/// The configured thread cap, for tests and diagnostics.
#[php_function]
#[php(name = "Phpolars\\threads")]
pub fn threads() -> i64 {
    init_threads();
    polars_core::runtime::THREAD_POOL.current_num_threads() as i64
}

/// Kept so deployment checks can confirm the extension loaded.
#[php_function]
#[php(name = "Phpolars\\version")]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

fn startup(_ty: i32, mod_num: i32) -> i32 {
    IniEntryDef::register(INI.as_slice(), mod_num);
    0
}

#[php_module]
#[php(startup = startup)]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    module
        .name("php_polars")
        .class::<PhpExpr>()
        .class::<PhpDataFrame>()
        .class::<PhpGroupBy>()
        .function(wrap_function!(col))
        .function(wrap_function!(lit))
        .function(wrap_function!(threads))
        .function(wrap_function!(version))
}
