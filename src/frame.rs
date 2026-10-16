use ext_php_rs::{
    boxed::ZBox,
    prelude::*,
    types::{ZendCallable, ZendHashTable, Zval},
};
use polars::prelude::*;
#[cfg(feature = "parquet")]
use std::fs::File;
use std::path::PathBuf;

use crate::{convert, expr::PhpExpr, init_threads, polars_error};

/// An eager Polars data frame. Every operation returns a new frame.
#[php_class]
#[php(name = "Phpolars\\DataFrame")]
pub struct PhpDataFrame {
    pub(crate) inner: DataFrame,
}

/// Grouping keys waiting for agg().
#[php_class]
#[php(name = "Phpolars\\GroupBy")]
pub struct PhpGroupBy {
    frame: DataFrame,
    keys: Vec<String>,
}

fn wrap(inner: DataFrame) -> PhpDataFrame {
    PhpDataFrame { inner }
}

fn collect(lf: LazyFrame) -> PhpResult<PhpDataFrame> {
    init_threads();
    lf.collect().map(wrap).map_err(polars_error)
}

fn strings(value: &Zval, what: &str) -> PhpResult<Vec<String>> {
    if let Some(s) = value.str() {
        return Ok(vec![s.to_owned()]);
    }
    let list = value
        .array()
        .ok_or_else(|| PhpException::from(format!("{what} must be a string or list of strings")))?;
    list.values()
        .map(|v| {
            v.str()
                .map(str::to_owned)
                .ok_or_else(|| PhpException::from(format!("{what} must be strings")))
        })
        .collect()
}

fn exprs(list: &ZendHashTable) -> PhpResult<Vec<Expr>> {
    list.values()
        .map(|v| {
            v.extract::<&PhpExpr>()
                .map(|e| e.inner.clone())
                .ok_or_else(|| PhpException::from("expected a list of Phpolars\\Expr"))
        })
        .collect()
}

fn check_columns(df: &DataFrame, names: &[String]) -> PhpResult<()> {
    for name in names {
        if df.column(name).is_err() {
            return Err(format!("column '{name}' not found").into());
        }
    }
    Ok(())
}

#[php_impl]
impl PhpDataFrame {
    pub fn read_csv(path: String) -> PhpResult<Self> {
        init_threads();
        CsvReadOptions::default()
            .with_has_header(true)
            .try_into_reader_with_file_path(Some(PathBuf::from(path)))
            .and_then(|r| r.finish())
            .map(wrap)
            .map_err(polars_error)
    }

    /// Errors when the extension was built without the parquet feature.
    pub fn read_parquet(path: String) -> PhpResult<Self> {
        init_threads();
        #[cfg(feature = "parquet")]
        {
            let file = File::open(&path).map_err(|e| PhpException::from(format!("{path}: {e}")))?;
            ParquetReader::new(file)
                .finish()
                .map(wrap)
                .map_err(polars_error)
        }
        #[cfg(not(feature = "parquet"))]
        Err(format!("{path}: php_polars was built without parquet support").into())
    }

    /// ['name' => [values...], ...]
    pub fn from_columns(columns: &ZendHashTable) -> PhpResult<Self> {
        convert::from_columns(columns).map(wrap)
    }

    /// [['name' => value, ...], ...]
    pub fn from_rows(rows: &ZendHashTable) -> PhpResult<Self> {
        convert::from_rows(rows).map(wrap)
    }

    /// Any Phibble\Contracts\Table, read through its toColumns().
    pub fn from_phibble(table: &Zval) -> PhpResult<Self> {
        let columns = table.try_call_method("toColumns", vec![])?;
        let columns = columns
            .array()
            .ok_or_else(|| PhpException::from("toColumns() must return an array"))?;
        convert::from_columns(columns).map(wrap)
    }

    pub fn columns(&self) -> Vec<String> {
        self.inner
            .get_column_names()
            .into_iter()
            .map(|n| n.to_string())
            .collect()
    }

    pub fn height(&self) -> i64 {
        self.inner.height() as i64
    }

    pub fn width(&self) -> i64 {
        self.inner.width() as i64
    }

    pub fn select(&self, names: &Zval) -> PhpResult<Self> {
        let names = strings(names, "select() names")?;
        check_columns(&self.inner, &names)?;
        self.inner.select(names).map(wrap).map_err(polars_error)
    }

    /// Keep rows where the predicate is true; null counts as false.
    pub fn filter(&self, predicate: &PhpExpr) -> PhpResult<Self> {
        collect(self.inner.clone().lazy().filter(predicate.inner.clone()))
    }

    /// Add or replace columns: ['name' => Expr, ...].
    pub fn with_columns(&self, columns: &ZendHashTable) -> PhpResult<Self> {
        let mut out = Vec::with_capacity(columns.len());
        for (key, v) in columns.iter() {
            let e = v
                .extract::<&PhpExpr>()
                .ok_or_else(|| PhpException::from("withColumns() values must be Phpolars\\Expr"))?;
            out.push(e.inner.clone().alias(key.to_string()));
        }
        collect(self.inner.clone().lazy().with_columns(out))
    }

    /// Stable sort; nulls last. $descending is a bool or one bool per key.
    pub fn sort(&self, by: &Zval, descending: Option<&Zval>) -> PhpResult<Self> {
        let by = strings(by, "sort() keys")?;
        check_columns(&self.inner, &by)?;
        let descending: Vec<bool> = match descending {
            None => vec![false],
            Some(d) => match (d.bool(), d.array()) {
                (Some(b), _) => vec![b],
                (_, Some(list)) => list.values().map(|v| v.bool().unwrap_or(false)).collect(),
                _ => return Err("sort() descending must be a bool or list of bools".into()),
            },
        };
        let options = SortMultipleOptions::default()
            .with_order_descending_multi(descending)
            .with_nulls_last(true)
            .with_maintain_order(true);
        collect(self.inner.clone().lazy().sort(by, options))
    }

    pub fn group_by(&self, keys: &Zval) -> PhpResult<PhpGroupBy> {
        let keys = strings(keys, "groupBy() keys")?;
        check_columns(&self.inner, &keys)?;
        Ok(PhpGroupBy {
            frame: self.inner.clone(),
            keys,
        })
    }

    #[php(defaults(n = 6))]
    pub fn head(&self, n: i64) -> Self {
        wrap(self.inner.head(Some(n.max(0) as usize)))
    }

    pub fn to_rows(&self) -> PhpResult<ZBox<ZendHashTable>> {
        convert::to_rows(&self.inner)
    }

    pub fn to_columns(&self) -> PhpResult<ZBox<ZendHashTable>> {
        convert::to_columns(&self.inner)
    }

    /// Requires phibble on the autoloader.
    pub fn to_phibble(&self) -> PhpResult<Zval> {
        let factory = ZendCallable::try_from_name("Phibble\\Frame::fromColumns")
            .map_err(|_| PhpException::from("Phibble\\Frame::fromColumns() is not available; install erikwestlund/phibble"))?;
        let columns = convert::to_columns(&self.inner)?;
        Ok(factory.try_call(vec![&columns])?)
    }

    pub fn __to_string(&self) -> String {
        format!("{}", self.inner)
    }
}

#[php_impl]
impl PhpGroupBy {
    /// One row per group, in order of first appearance: keys, then aggregates.
    pub fn agg(&self, aggregates: &ZendHashTable) -> PhpResult<PhpDataFrame> {
        let keys: Vec<Expr> = self.keys.iter().map(|k| col(k.as_str())).collect();
        collect(
            self.frame
                .clone()
                .lazy()
                .group_by_stable(keys)
                .agg(exprs(aggregates)?),
        )
    }
}
