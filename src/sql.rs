#![allow(non_snake_case)]
//! SQL over frames, as py-polars' SQLContext, DataFrame.sql(), and pl.sql().
//! Built only with the `sql` Cargo feature; without it these raise a ComputeError.
use ext_php_rs::{prelude::*, types::ZendHashTable, types::Zval};
use polars::prelude::*;

use crate::{
    error::OrPhp,
    frame::{PhpDataFrame, PhpLazyFrame},
};

#[cfg(not(feature = "sql"))]
fn missing() -> PhpException {
    PhpException::from_class::<crate::error::ComputeError>(
        "php_polars was built without the 'sql' feature".to_string(),
    )
}

/// A DataFrame or LazyFrame argument as a LazyFrame.
fn lazy(v: &Zval) -> PhpResult<LazyFrame> {
    if let Some(df) = v.extract::<&PhpDataFrame>() {
        return Ok(df.inner.clone().lazy());
    }
    if let Some(lf) = v.extract::<&PhpLazyFrame>() {
        return Ok(lf.inner.clone());
    }
    Err(crate::error::value_error(
        "expected a Phpolars\\DataFrame or Phpolars\\LazyFrame",
    ))
}

/// The query result: a LazyFrame, or a collected DataFrame when eager.
fn result(lf: LazyFrame, eager: bool) -> PhpResult<Zval> {
    use ext_php_rs::convert::IntoZval;
    if eager {
        crate::init_threads();
        Ok(PhpDataFrame::wrap(lf.collect().php()?).into_zval(false)?)
    } else {
        Ok(PhpLazyFrame::wrap(lf).into_zval(false)?)
    }
}

/// Run one query over named frames.
pub fn run(query: &str, frames: &[(String, LazyFrame)]) -> PhpResult<LazyFrame> {
    #[cfg(feature = "sql")]
    {
        let mut ctx = polars::sql::SQLContext::new();
        for (name, lf) in frames {
            ctx.register(name, lf.clone());
        }
        ctx.execute(query).php()
    }
    #[cfg(not(feature = "sql"))]
    {
        let _ = (query, frames);
        Err(missing())
    }
}

fn named_frames(frames: Option<&ZendHashTable>) -> PhpResult<Vec<(String, LazyFrame)>> {
    let mut out = Vec::new();
    if let Some(f) = frames {
        for (k, v) in f.iter() {
            out.push((k.to_string(), lazy(v)?));
        }
    }
    Ok(out)
}

/// new SQLContext(['sales' => $df]); $ctx->execute('SELECT ...').
/// execute() returns a LazyFrame unless eager is set, as in py.
#[php_class]
#[php(name = "Phpolars\\SQLContext")]
pub struct PhpSqlContext {
    frames: Vec<(String, LazyFrame)>,
    eager: bool,
}

#[php_impl]
impl PhpSqlContext {
    pub fn __construct(frames: Option<&ZendHashTable>, eager: Option<bool>) -> PhpResult<Self> {
        Ok(Self {
            frames: named_frames(frames)?,
            eager: eager.unwrap_or(false),
        })
    }

    /// Register a frame under a table name, replacing any with that name.
    pub fn register(&mut self, name: String, frame: &Zval) -> PhpResult<()> {
        let lf = lazy(frame)?;
        self.frames.retain(|(n, _)| n != &name);
        self.frames.push((name, lf));
        Ok(())
    }

    /// Register several frames: registerMany(['a' => $df, 'b' => $lf]).
    pub fn registerMany(&mut self, frames: &ZendHashTable) -> PhpResult<()> {
        for (name, lf) in named_frames(Some(frames))? {
            self.frames.retain(|(n, _)| n != &name);
            self.frames.push((name, lf));
        }
        Ok(())
    }

    pub fn unregister(&mut self, names: &Zval) -> PhpResult<()> {
        let names = crate::input::names(&[names])?;
        self.frames.retain(|(n, _)| !names.contains(n));
        Ok(())
    }

    /// The registered table names, sorted.
    pub fn tables(&self) -> Vec<String> {
        let mut t: Vec<String> = self.frames.iter().map(|(n, _)| n.clone()).collect();
        t.sort();
        t
    }

    pub fn execute(&self, query: String, eager: Option<bool>) -> PhpResult<Zval> {
        result(run(&query, &self.frames)?, eager.unwrap_or(self.eager))
    }
}

/// sql('SELECT * FROM t', ['t' => $df]): one query over named frames.
/// Returns a LazyFrame unless eager: true.
#[php_function]
#[php(name = "Phpolars\\sql")]
pub fn sql(query: String, frames: Option<&ZendHashTable>, eager: Option<bool>) -> PhpResult<Zval> {
    result(run(&query, &named_frames(frames)?)?, eager.unwrap_or(false))
}

pub fn register(module: ModuleBuilder) -> ModuleBuilder {
    module
        .class::<PhpSqlContext>()
        .function(wrap_function!(sql))
}
