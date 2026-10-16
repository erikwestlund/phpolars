#![allow(non_snake_case)]
//! Phpolars\DataFrame, Phpolars\LazyFrame and their GroupBy classes.
//!
//! The verbs both frames share are written once in `frame_verbs!`: each one
//! builds on a LazyFrame, and DataFrame collects the result while LazyFrame
//! keeps it lazy, exactly as py-polars does.
use ext_php_rs::{
    boxed::ZBox,
    prelude::*,
    types::{ZendCallable, ZendHashTable, Zval},
};
use polars::prelude::*;

use crate::{
    convert, dtype,
    error::{OrPhp, column_not_found, value_error},
    expr::fill_null,
    init_threads,
    input::{bools, exprs, given, into_expr, names, operand, opt_names},
    series::PhpSeries,
};

#[php_class]
#[php(name = "Phpolars\\DataFrame")]
#[derive(Clone)]
pub struct PhpDataFrame {
    pub(crate) inner: DataFrame,
}

#[php_class]
#[php(name = "Phpolars\\LazyFrame")]
#[derive(Clone)]
pub struct PhpLazyFrame {
    pub(crate) inner: LazyFrame,
}

impl PhpDataFrame {
    pub fn wrap(inner: DataFrame) -> Self {
        Self { inner }
    }

    fn lf(&self) -> LazyFrame {
        self.inner.clone().lazy()
    }

    fn out(lf: LazyFrame) -> PhpResult<Self> {
        init_threads();
        lf.collect().php().map(Self::wrap)
    }

    fn schema_ref(&self) -> PhpResult<SchemaRef> {
        Ok(self.inner.schema().clone())
    }
}

impl PhpLazyFrame {
    pub fn wrap(inner: LazyFrame) -> Self {
        Self { inner }
    }

    fn lf(&self) -> LazyFrame {
        self.inner.clone()
    }

    fn out(lf: LazyFrame) -> PhpResult<Self> {
        Ok(Self::wrap(lf))
    }

    fn schema_ref(&self) -> PhpResult<SchemaRef> {
        self.inner.clone().collect_schema().php()
    }
}

fn join_type(how: Option<String>) -> PhpResult<JoinType> {
    Ok(match how.as_deref().unwrap_or("inner") {
        "inner" => JoinType::Inner,
        "left" => JoinType::Left,
        "right" => JoinType::Right,
        "full" | "outer" => JoinType::Full,
        "semi" => JoinType::Semi,
        "anti" => JoinType::Anti,
        "cross" => JoinType::Cross,
        other => return Err(value_error(format!("unknown join type '{other}'"))),
    })
}

fn unique_keep(keep: Option<String>) -> PhpResult<UniqueKeepStrategy> {
    Ok(match keep.as_deref().unwrap_or("any") {
        "any" => UniqueKeepStrategy::Any,
        "first" => UniqueKeepStrategy::First,
        "last" => UniqueKeepStrategy::Last,
        "none" => UniqueKeepStrategy::None,
        other => {
            return Err(value_error(format!(
                "keep must be any, first, last or none, not '{other}'"
            )));
        }
    })
}

fn sort_options(
    descending: Option<&Zval>,
    nullsLast: Option<&Zval>,
    maintainOrder: Option<bool>,
) -> PhpResult<SortMultipleOptions> {
    Ok(SortMultipleOptions::default()
        .with_order_descending_multi(bools(descending, false)?)
        .with_nulls_last_multi(bools(nullsLast, false)?)
        .with_maintain_order(maintainOrder.unwrap_or(false)))
}

/// py filter(*predicates): Exprs, column names of booleans, or boolean lists.
fn all_of(args: &[&Zval]) -> PhpResult<Expr> {
    let mut all: Option<Expr> = None;
    for e in exprs(args)? {
        all = Some(match all {
            None => e,
            Some(acc) => acc.and(e),
        });
    }
    all.ok_or_else(|| value_error("filter() needs at least one predicate"))
}

/// Strategy-free frame aggregates such as sum() and mean().
fn frame_agg(lf: LazyFrame, f: impl Fn(Expr) -> Expr) -> LazyFrame {
    lf.select([f(all().as_expr())])
}

/// The column-name argument of drop()/unique()/dropNulls() as a selector.
fn selector(cols: Vec<String>) -> Selector {
    by_name(cols, true, false)
}

macro_rules! frame_verbs {
    ($ty:ident, $group:ident, { $($extra:tt)* }) => {
        #[php_impl]
        impl $ty {
            $($extra)*

            /// select('a', col('b')->mul(2), ['c' => col('x')])
            pub fn select(&self, exprs: &[&Zval]) -> PhpResult<Self> {
                Self::out(self.lf().select(crate::input::exprs(exprs)?))
            }

            pub fn withColumns(&self, exprs: &[&Zval]) -> PhpResult<Self> {
                Self::out(self.lf().with_columns(crate::input::exprs(exprs)?))
            }

            /// Keep rows where every predicate is true; null counts as false.
            pub fn filter(&self, predicates: &[&Zval]) -> PhpResult<Self> {
                Self::out(self.lf().filter(all_of(predicates)?))
            }

            /// sort('a', ['b', 'c'], descending: [false, true], nullsLast: true)
            pub fn sort(&self, by: &Zval, descending: Option<&Zval>, nullsLast: Option<&Zval>, maintainOrder: Option<bool>) -> PhpResult<Self> {
                let by = exprs(&[by])?;
                Self::out(self.lf().sort_by_exprs(by, sort_options(descending, nullsLast, maintainOrder)?))
            }

            pub fn head(&self, n: Option<i64>) -> PhpResult<Self> {
                Self::out(self.lf().limit(n.unwrap_or(5).max(0) as IdxSize))
            }

            pub fn limit(&self, n: Option<i64>) -> PhpResult<Self> {
                self.head(n)
            }

            pub fn tail(&self, n: Option<i64>) -> PhpResult<Self> {
                Self::out(self.lf().tail(n.unwrap_or(5).max(0) as IdxSize))
            }

            pub fn slice(&self, offset: i64, length: Option<i64>) -> PhpResult<Self> {
                let len = length.map(|l| l.max(0) as IdxSize).unwrap_or(IdxSize::MAX);
                Self::out(self.lf().slice(offset, len))
            }

            pub fn first(&self) -> PhpResult<Self> {
                Self::out(self.lf().first())
            }

            pub fn last(&self) -> PhpResult<Self> {
                Self::out(self.lf().last())
            }

            pub fn reverse(&self) -> PhpResult<Self> {
                Self::out(self.lf().reverse())
            }

            /// drop('a', 'b'); unknown names are an error, as py's strict=True.
            pub fn drop(&self, columns: &[&Zval]) -> PhpResult<Self> {
                let cols = names(columns)?;
                {
                    let schema = self.schema_ref()?;
                    for c in &cols {
                        if schema.get(c.as_str()).is_none() {
                            return Err(column_not_found(c));
                        }
                    }
                }
                Self::out(self.lf().drop(selector(cols)))
            }

            /// rename(['old' => 'new', ...]), as py's rename({'old': 'new'}).
            pub fn rename(&self, mapping: &ZendHashTable, strict: Option<bool>) -> PhpResult<Self> {
                let mut old = Vec::new();
                let mut new = Vec::new();
                for (k, v) in mapping.iter() {
                    old.push(k.to_string());
                    new.push(v.str().ok_or_else(|| value_error("rename() values must be strings"))?.to_owned());
                }
                Self::out(self.lf().rename(old, new, strict.unwrap_or(true)))
            }

            pub fn unique(&self, subset: Option<&Zval>, keep: Option<String>, maintainOrder: Option<bool>) -> PhpResult<Self> {
                let subset = opt_names(subset)?.map(selector);
                let keep = unique_keep(keep)?;
                Self::out(if maintainOrder.unwrap_or(false) {
                    self.lf().unique_stable(subset, keep)
                } else {
                    self.lf().unique(subset, keep)
                })
            }

            /// A value fills only the columns it fits, as in py: numbers fill
            /// numeric columns, strings fill String columns. A strategy fills all.
            pub fn fillNull(&self, value: Option<&Zval>, strategy: Option<String>, limit: Option<i64>) -> PhpResult<Self> {
                let schema = self.schema_ref()?;
                let fits = |dt: &DataType| -> bool {
                    match given(value) {
                        None => true,
                        Some(v) if v.is_long() || v.is_double() => dt.is_primitive_numeric() || matches!(dt, DataType::Decimal(..)),
                        Some(v) if v.is_string() => dt.is_string() || matches!(dt, DataType::Categorical(..)),
                        Some(v) if v.is_bool() => dt.is_bool(),
                        Some(_) => true,
                    }
                };
                let mut exprs = Vec::new();
                for (name, dt) in schema.iter() {
                    if dt.is_null() || fits(dt) {
                        exprs.push(fill_null(col(name.clone()), value, strategy.clone(), limit)?);
                    }
                }
                Self::out(self.lf().with_columns(exprs))
            }

            pub fn fillNan(&self, value: &Zval) -> PhpResult<Self> {
                Self::out(self.lf().fill_nan(operand(value)?))
            }

            pub fn dropNulls(&self, subset: Option<&Zval>) -> PhpResult<Self> {
                Self::out(self.lf().drop_nulls(opt_names(subset)?.map(selector)))
            }

            pub fn nullCount(&self) -> PhpResult<Self> {
                Self::out(self.lf().null_count())
            }

            pub fn withRowIndex(&self, name: Option<String>, offset: Option<i64>) -> PhpResult<Self> {
                let name = name.unwrap_or_else(|| "index".into());
                Self::out(self.lf().with_row_index(name, offset.map(|o| o as IdxSize)))
            }

            /// cast(['a' => 'Int64']) or cast('Float64') for every column.
            pub fn cast(&self, dtypes: &Zval, strict: Option<bool>) -> PhpResult<Self> {
                let strict = strict.unwrap_or(true);
                if let Some(d) = dtypes.str() {
                    return Self::out(self.lf().cast_all(dtype::parse(d)?, strict));
                }
                let map = dtypes.array().ok_or_else(|| value_error("cast() takes a type name or ['column' => type]"))?;
                let parsed = convert::schema_overrides(Some(map))?;
                let schema = self.schema_ref()?;
                for (c, _) in &parsed {
                    if schema.get(c.as_str()).is_none() {
                        return Err(column_not_found(c));
                    }
                }
                let m: PlHashMap<&str, DataType> = parsed.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
                Self::out(self.lf().cast(m, strict))
            }

            pub fn explode(&self, columns: &[&Zval]) -> PhpResult<Self> {
                Self::out(self.lf().explode(selector(names(columns)?), ExplodeOptions { empty_as_null: true, keep_nulls: true }))
            }

            pub fn unnest(&self, columns: &[&Zval]) -> PhpResult<Self> {
                Self::out(self.lf().unnest(selector(names(columns)?), None))
            }

            /// join($other, on: 'id', how: 'left') or leftOn/rightOn; how is
            /// inner, left, right, full, semi, anti or cross.
            pub fn join(&self, other: &Zval, on: Option<&Zval>, how: Option<String>, leftOn: Option<&Zval>, rightOn: Option<&Zval>, suffix: Option<String>, coalesce: Option<bool>, maintainOrder: Option<String>, nullsEqual: Option<bool>) -> PhpResult<Self> {
                let other = other_frame(other)?;
                let how = join_type(how)?;
                let (left, right) = match (given(on), given(leftOn), given(rightOn)) {
                    (Some(on), None, None) => {
                        let e = exprs(&[on])?;
                        (e.clone(), e)
                    }
                    (_, Some(l), Some(r)) => (exprs(&[l])?, exprs(&[r])?),
                    _ if matches!(how, JoinType::Cross) => (vec![], vec![]),
                    _ => return Err(value_error("join() needs on, or both leftOn and rightOn")),
                };
                let coalesce = match coalesce {
                    None => JoinCoalesce::JoinSpecific,
                    Some(true) => JoinCoalesce::CoalesceColumns,
                    Some(false) => JoinCoalesce::KeepColumns,
                };
                let order = match maintainOrder.as_deref().unwrap_or("none") {
                    "none" => MaintainOrderJoin::None,
                    "left" => MaintainOrderJoin::Left,
                    "right" => MaintainOrderJoin::Right,
                    "left_right" => MaintainOrderJoin::LeftRight,
                    "right_left" => MaintainOrderJoin::RightLeft,
                    o => return Err(value_error(format!("unknown maintainOrder '{o}'"))),
                };
                let mut args = JoinArgs::new(how)
                    .with_suffix(Some(suffix.unwrap_or_else(|| "_right".into()).into()))
                    .with_coalesce(coalesce);
                args.maintain_order = order;
                args.nulls_equal = nullsEqual.unwrap_or(false);
                Self::out(self.lf().join(other, left, right, args))
            }

            /// groupBy('a'), groupBy(['a', col('b')], maintainOrder: true)
            pub fn groupBy(&self, by: &Zval, maintainOrder: Option<bool>) -> PhpResult<$group> {
                let keys = exprs(&[by])?;
                if keys.is_empty() {
                    return Err(value_error("groupBy() needs at least one key"));
                }
                Ok($group {
                    lf: self.lf(),
                    keys,
                    maintain_order: maintainOrder.unwrap_or(false),
                })
            }

            pub fn sum(&self) -> PhpResult<Self> {
                Self::out(self.lf().sum())
            }

            pub fn mean(&self) -> PhpResult<Self> {
                Self::out(self.lf().mean())
            }

            pub fn median(&self) -> PhpResult<Self> {
                Self::out(self.lf().median())
            }

            pub fn min(&self) -> PhpResult<Self> {
                Self::out(self.lf().min())
            }

            pub fn max(&self) -> PhpResult<Self> {
                Self::out(self.lf().max())
            }

            pub fn std(&self, ddof: Option<i64>) -> PhpResult<Self> {
                Self::out(self.lf().std(ddof.unwrap_or(1) as u8))
            }

            pub fn var(&self, ddof: Option<i64>) -> PhpResult<Self> {
                Self::out(self.lf().var(ddof.unwrap_or(1) as u8))
            }

            pub fn count(&self) -> PhpResult<Self> {
                Self::out(frame_agg(self.lf(), |e| e.count()))
            }

            pub fn nUnique(&self) -> PhpResult<Self> {
                Self::out(frame_agg(self.lf(), |e| e.n_unique()))
            }

            /// unpivot(on: ['x', 'y'], index: 'id', variableName:, valueName:)
            pub fn unpivot(&self, on: Option<&Zval>, index: Option<&Zval>, variableName: Option<String>, valueName: Option<String>) -> PhpResult<Self> {
                let index = opt_names(index)?.unwrap_or_default();
                let on = match opt_names(on)? {
                    Some(on) => on,
                    None => self
                        .schema_ref()?
                        .iter_names()
                        .map(|n| n.to_string())
                        .filter(|n| !index.contains(n))
                        .collect(),
                };
                let args = UnpivotArgsDSL {
                    on: Some(selector(on)),
                    index: selector(index),
                    variable_name: variableName.map(Into::into),
                    value_name: valueName.map(Into::into),
                };
                Self::out(self.lf().unpivot(args))
            }

            /// Columns and their py data type names.
            pub fn collectSchema(&self) -> PhpResult<Vec<(String, String)>> {
                Ok(dtype::schema_array(self.schema_ref()?.as_ref()))
            }
        }
    };
}

/// A DataFrame or LazyFrame argument as a LazyFrame.
fn other_frame(other: &Zval) -> PhpResult<LazyFrame> {
    if let Some(df) = other.extract::<&PhpDataFrame>() {
        return Ok(df.inner.clone().lazy());
    }
    if let Some(lf) = other.extract::<&PhpLazyFrame>() {
        return Ok(lf.inner.clone());
    }
    Err(value_error(
        "expected a Phpolars\\DataFrame or Phpolars\\LazyFrame",
    ))
}

frame_verbs!(PhpDataFrame, PhpGroupBy, {
    /// new DataFrame(['a' => [1, 2], 'b' => ['x', 'y']]) or a list of rows.
    pub fn __construct(
        data: Option<&ZendHashTable>,
        schema: Option<&ZendHashTable>,
    ) -> PhpResult<Self> {
        let df = match data {
            None => convert::from_columns(&ZendHashTable::new(), schema)?,
            Some(d) if convert::is_list(d) && !d.is_empty() => convert::from_rows(d, schema)?,
            Some(d) => convert::from_columns(d, schema)?,
        };
        Ok(Self::wrap(df))
    }

    /// ['name' => [values...] | Series, ...]
    pub fn fromDict(data: &ZendHashTable, schema: Option<&ZendHashTable>) -> PhpResult<Self> {
        convert::from_columns(data, schema).map(Self::wrap)
    }

    /// [['name' => value, ...], ...]
    pub fn fromDicts(data: &ZendHashTable, schema: Option<&ZendHashTable>) -> PhpResult<Self> {
        convert::from_rows(data, schema).map(Self::wrap)
    }

    /// [[v1, v2, ...], ...] with the schema naming the columns.
    pub fn fromRows(data: &ZendHashTable, schema: Option<&ZendHashTable>) -> PhpResult<Self> {
        convert::from_rows(data, schema).map(Self::wrap)
    }

    /// Any Phibble\Contracts\Table, read through its toColumns().
    pub fn fromPhibble(table: &Zval) -> PhpResult<Self> {
        let columns = table.try_call_method("toColumns", vec![])?;
        let columns = columns
            .array()
            .ok_or_else(|| value_error("toColumns() must return an array"))?;
        convert::from_columns(columns, None).map(Self::wrap)
    }

    pub fn lazy(&self) -> PhpLazyFrame {
        PhpLazyFrame::wrap(self.lf())
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

    /// [height, width]
    pub fn shape(&self) -> Vec<i64> {
        vec![self.inner.height() as i64, self.inner.width() as i64]
    }

    pub fn isEmpty(&self) -> bool {
        self.inner.height() == 0
    }

    /// ['column' => 'Int64', ...]
    pub fn schema(&self) -> Vec<(String, String)> {
        dtype::schema_array(self.inner.schema())
    }

    pub fn dtypes(&self) -> Vec<String> {
        self.inner
            .columns()
            .iter()
            .map(|c| dtype::name(c.dtype()))
            .collect()
    }

    pub fn estimatedSize(&self) -> i64 {
        self.inner.estimated_size() as i64
    }

    pub fn getColumn(&self, name: String) -> PhpResult<PhpSeries> {
        let c = self
            .inner
            .column(&name)
            .map_err(|_| column_not_found(&name))?;
        Ok(PhpSeries {
            inner: c.as_materialized_series().clone(),
        })
    }

    pub fn getColumns(&self) -> Vec<PhpSeries> {
        self.inner
            .columns()
            .iter()
            .map(|c| PhpSeries {
                inner: c.as_materialized_series().clone(),
            })
            .collect()
    }

    pub fn toSeries(&self, index: Option<i64>) -> PhpResult<PhpSeries> {
        let w = self.inner.width() as i64;
        let i = index.unwrap_or(0);
        let i = if i < 0 { w + i } else { i };
        if i < 0 || i >= w {
            return Err(PhpException::from_class::<crate::error::OutOfBoundsError>(
                format!("column index {i} is out of bounds"),
            ));
        }
        Ok(PhpSeries {
            inner: self.inner.columns()[i as usize]
                .as_materialized_series()
                .clone(),
        })
    }

    /// [['col' => value, ...], ...]
    pub fn toDicts(&self) -> PhpResult<ZBox<ZendHashTable>> {
        convert::to_rows(&self.inner)
    }

    /// [[v1, v2, ...], ...], or keyed rows with named: true.
    pub fn rows(&self, named: Option<bool>) -> PhpResult<ZBox<ZendHashTable>> {
        if named.unwrap_or(false) {
            convert::to_rows(&self.inner)
        } else {
            convert::to_tuples(&self.inner)
        }
    }

    /// One row by position, as a list or (named: true) keyed by column.
    pub fn row(&self, index: i64, named: Option<bool>) -> PhpResult<Zval> {
        let h = self.inner.height() as i64;
        let i = if index < 0 { h + index } else { index };
        if i < 0 || i >= h {
            return Err(PhpException::from_class::<crate::error::OutOfBoundsError>(
                format!("row index {index} is out of bounds"),
            ));
        }
        let one = self.inner.slice(i, 1);
        let rows = if named.unwrap_or(false) {
            convert::to_rows(&one)?
        } else {
            convert::to_tuples(&one)?
        };
        Ok(rows
            .get_index(0)
            .map(|z| z.shallow_clone())
            .unwrap_or_default())
    }

    /// The single value of a 1x1 frame, or the value at (row, column).
    pub fn item(&self, row: Option<i64>, column: Option<&Zval>) -> PhpResult<Zval> {
        let (r, c) = match (row, given(column)) {
            (None, None) => {
                if self.inner.shape() != (1, 1) {
                    return Err(value_error(format!(
                        "item() without arguments needs a 1x1 frame, not {:?}",
                        self.inner.shape()
                    )));
                }
                (0, 0)
            }
            (Some(r), Some(c)) => {
                let ci = match c.str() {
                    Some(name) => self
                        .inner
                        .get_column_index(name)
                        .ok_or_else(|| column_not_found(name))?,
                    None => c
                        .long()
                        .ok_or_else(|| value_error("column must be a name or index"))?
                        as usize,
                };
                (r, ci)
            }
            _ => return Err(value_error("item() takes both row and column, or neither")),
        };
        let s = self.toSeries(Some(c as i64))?;
        s.get(r)
    }

    /// ['column' => Series], or lists of values with asSeries: false.
    pub fn toDict(&self, asSeries: Option<bool>) -> PhpResult<ZBox<ZendHashTable>> {
        if !asSeries.unwrap_or(true) {
            return convert::to_columns(&self.inner);
        }
        let mut out = ZendHashTable::with_capacity(self.inner.width() as u32);
        for c in self.inner.columns() {
            out.insert(
                c.name().as_str(),
                PhpSeries {
                    inner: c.as_materialized_series().clone(),
                },
            )?;
        }
        Ok(out)
    }

    /// ['column' => [values...]]: toDict(asSeries: false).
    pub fn toColumns(&self) -> PhpResult<ZBox<ZendHashTable>> {
        convert::to_columns(&self.inner)
    }

    /// A Phibble\Frame; requires erikwestlund/phibble.
    pub fn toPhibble(&self) -> PhpResult<Zval> {
        let factory = ZendCallable::try_from_name("Phibble\\Frame::fromColumns").map_err(|_| {
            PhpException::from(
                "Phibble\\Frame::fromColumns() is not available; install erikwestlund/phibble",
            )
        })?;
        let columns = convert::to_columns(&self.inner)?;
        Ok(factory.try_call(vec![&columns])?)
    }

    /// Summary statistics per column, as py's describe().
    pub fn describe(&self, percentiles: Option<Vec<crate::input::Num>>) -> PhpResult<Self> {
        describe(
            &self.inner,
            percentiles.map_or_else(
                || vec![0.25, 0.5, 0.75],
                |p| p.into_iter().map(|n| n.0).collect(),
            ),
        )
    }

    /// pivot(on: 'k', index: 'id', values: 'v', aggregateFunction: 'sum')
    pub fn pivot(
        &self,
        on: &Zval,
        index: Option<&Zval>,
        values: Option<&Zval>,
        aggregateFunction: Option<&Zval>,
        sortColumns: Option<bool>,
        separator: Option<String>,
        maintainOrder: Option<bool>,
    ) -> PhpResult<Self> {
        let on = names(&[on])?;
        let schema = self.inner.schema();
        let index = opt_names(index)?;
        let values = opt_names(values)?;
        let all_names: Vec<String> = schema.iter_names().map(|n| n.to_string()).collect();
        let (index, values) = match (index, values) {
            (Some(i), Some(v)) => (i, v),
            (Some(i), None) => {
                let v = all_names
                    .iter()
                    .filter(|n| !on.contains(n) && !i.contains(n))
                    .cloned()
                    .collect();
                (i, v)
            }
            (None, Some(v)) => {
                let i = all_names
                    .iter()
                    .filter(|n| !on.contains(n) && !v.contains(n))
                    .cloned()
                    .collect();
                (i, v)
            }
            (None, None) => return Err(value_error("pivot() needs index or values")),
        };
        let agg = match given(aggregateFunction) {
            None => element().first(),
            Some(f) => match f.str() {
                Some("first") => element().first(),
                Some("last") => element().last(),
                Some("sum") => element().sum(),
                Some("mean") => element().mean(),
                Some("median") => element().median(),
                Some("min") => element().min(),
                Some("max") => element().max(),
                Some("len") => element().len(),
                Some(other) => {
                    return Err(value_error(format!("unknown aggregate function '{other}'")));
                }
                None => into_expr(f)?,
            },
        };
        let mut on_columns = self
            .inner
            .select(on.iter().map(|s| s.as_str()))
            .php()?
            .unique_stable(None, UniqueKeepStrategy::First, None)
            .php()?;
        if sortColumns.unwrap_or(false) {
            on_columns = on_columns
                .sort(on.clone(), SortMultipleOptions::default())
                .php()?;
        }
        let lf = self.lf().pivot(
            selector(on),
            std::sync::Arc::new(on_columns),
            selector(index),
            selector(values),
            agg,
            maintainOrder.unwrap_or(true),
            separator.unwrap_or_else(|| "_".into()).into(),
            polars_core::frame::PivotColumnNaming::Auto,
        );
        Self::out(lf)
    }

    /// Rows become columns. includeHeader keeps the old names in headerName.
    pub fn transpose(
        &self,
        includeHeader: Option<bool>,
        headerName: Option<String>,
        columnNames: Option<&Zval>,
    ) -> PhpResult<Self> {
        let header = headerName.unwrap_or_else(|| "column".into());
        let keep = if includeHeader.unwrap_or(false) {
            Some(header.as_str())
        } else {
            None
        };
        let new_names = match given(columnNames) {
            None => None,
            Some(v) => match v.str() {
                Some(c) => Some(either::Either::Left(c.to_string())),
                None => Some(either::Either::Right(names(&[v])?)),
            },
        };
        let mut df = self.inner.clone();
        df.transpose(keep, new_names).php().map(Self::wrap)
    }

    pub fn sample(
        &self,
        n: Option<i64>,
        fraction: Option<crate::input::Num>,
        withReplacement: Option<bool>,
        shuffle: Option<bool>,
        seed: Option<i64>,
    ) -> PhpResult<Self> {
        let wr = withReplacement.unwrap_or(false);
        let seed = seed.map(|s| s as u64);
        let out = match (n, fraction) {
            (Some(_), Some(_)) => {
                return Err(value_error("sample() takes n or fraction, not both"));
            }
            (_, Some(f)) => {
                self.inner
                    .sample_frac(&Series::new("frac".into(), [f.0]), wr, shuffle, seed)
            }
            (n, None) => {
                self.inner
                    .sample_n_literal(n.unwrap_or(1).max(0) as usize, wr, shuffle, seed)
            }
        };
        out.php().map(Self::wrap)
    }

    pub fn writeCsv(
        &self,
        file: String,
        includeHeader: Option<bool>,
        separator: Option<String>,
        nullValue: Option<String>,
    ) -> PhpResult<()> {
        crate::io::sink_csv(self.lf(), &file, includeHeader, separator, nullValue)
    }

    pub fn writeParquet(&self, file: String) -> PhpResult<()> {
        crate::io::sink_parquet(self.lf(), &file)
    }

    /// Arrow IPC (Feather v2).
    pub fn writeIpc(&self, file: String) -> PhpResult<()> {
        crate::io::sink_ipc(self.lf(), &file)
    }

    pub fn writeNdjson(&self, file: String) -> PhpResult<()> {
        crate::io::sink_ndjson(self.lf(), &file)
    }

    /// A JSON array of row objects.
    pub fn writeJson(&self, file: String) -> PhpResult<()> {
        crate::io::write_json(&self.inner, &file)
    }

    pub fn equals(&self, other: &PhpDataFrame, nullEqual: Option<bool>) -> bool {
        if nullEqual.unwrap_or(true) {
            self.inner.equals_missing(&other.inner)
        } else {
            self.inner.equals(&other.inner)
        }
    }

    pub fn clone(&self) -> Self {
        Self::wrap(self.inner.clone())
    }

    /// Add Series or columns from another frame: hstack([$s1, $s2]).
    pub fn hstack(&self, columns: &Zval) -> PhpResult<Self> {
        let mut add: Vec<Column> = Vec::new();
        if let Some(df) = columns.extract::<&PhpDataFrame>() {
            add.extend(df.inner.columns().iter().cloned());
        } else if let Some(list) = columns.array() {
            for v in list.values() {
                let s = v
                    .extract::<&PhpSeries>()
                    .ok_or_else(|| value_error("hstack() takes a DataFrame or a list of Series"))?;
                add.push(s.inner.clone().into());
            }
        } else {
            return Err(value_error(
                "hstack() takes a DataFrame or a list of Series",
            ));
        }
        self.inner.hstack(&add).php().map(Self::wrap)
    }

    /// Append the rows of another frame with the same schema.
    pub fn vstack(&self, other: &PhpDataFrame) -> PhpResult<Self> {
        self.inner.vstack(&other.inner).php().map(Self::wrap)
    }

    pub fn __to_string(&self) -> String {
        format!("{}", self.inner)
    }
});

frame_verbs!(PhpLazyFrame, PhpLazyGroupBy, {
    pub fn collect(&self) -> PhpResult<PhpDataFrame> {
        init_threads();
        self.inner.clone().collect().php().map(PhpDataFrame::wrap)
    }

    /// The query plan, optimized unless optimized: false.
    pub fn explain(&self, optimized: Option<bool>) -> PhpResult<String> {
        self.inner.explain(optimized.unwrap_or(true)).php()
    }

    pub fn columns(&self) -> PhpResult<Vec<String>> {
        Ok(self
            .schema_ref()?
            .iter_names()
            .map(|n| n.to_string())
            .collect())
    }

    pub fn sinkCsv(
        &self,
        file: String,
        includeHeader: Option<bool>,
        separator: Option<String>,
        nullValue: Option<String>,
    ) -> PhpResult<()> {
        crate::io::sink_csv(self.lf(), &file, includeHeader, separator, nullValue)
    }

    pub fn sinkParquet(&self, file: String) -> PhpResult<()> {
        crate::io::sink_parquet(self.lf(), &file)
    }

    pub fn sinkIpc(&self, file: String) -> PhpResult<()> {
        crate::io::sink_ipc(self.lf(), &file)
    }

    pub fn sinkNdjson(&self, file: String) -> PhpResult<()> {
        crate::io::sink_ndjson(self.lf(), &file)
    }

    pub fn cache(&self) -> Self {
        Self::wrap(self.inner.clone().cache())
    }

    /// Collect only the first n rows, for a quick look at a plan.
    pub fn fetch(&self, n: Option<i64>) -> PhpResult<PhpDataFrame> {
        init_threads();
        self.inner
            .clone()
            .limit(n.unwrap_or(500) as IdxSize)
            .collect()
            .php()
            .map(PhpDataFrame::wrap)
    }

    pub fn __to_string(&self) -> String {
        self.inner.explain(false).unwrap_or_default()
    }
});

macro_rules! group_by {
    ($ty:ident, $php:literal, $frame:ident) => {
        #[php_class]
        #[php(name = $php)]
        pub struct $ty {
            lf: LazyFrame,
            keys: Vec<Expr>,
            maintain_order: bool,
        }

        impl $ty {
            fn grouped(&self) -> LazyGroupBy {
                if self.maintain_order {
                    self.lf.clone().group_by_stable(self.keys.clone())
                } else {
                    self.lf.clone().group_by(self.keys.clone())
                }
            }

            fn all_other(&self, f: impl Fn(Expr) -> Expr) -> PhpResult<$frame> {
                let keys: Vec<PlSmallStr> = self
                    .keys
                    .iter()
                    .filter_map(|k| k.clone().meta().output_name().ok())
                    .collect();
                let rest = all().exclude_cols(keys).as_expr();
                $frame::out(self.grouped().agg([f(rest)]))
            }
        }

        #[php_impl]
        impl $ty {
            /// agg(col('x')->sum(), ['n' => col('y')->count()])
            pub fn agg(&self, aggs: &[&Zval]) -> PhpResult<$frame> {
                $frame::out(self.grouped().agg(exprs(aggs)?))
            }

            pub fn len(&self, name: Option<String>) -> PhpResult<$frame> {
                $frame::out(
                    self.grouped()
                        .agg([pl_len().alias(name.unwrap_or_else(|| "len".into()))]),
                )
            }

            pub fn sum(&self) -> PhpResult<$frame> {
                self.all_other(|e| e.sum())
            }

            pub fn mean(&self) -> PhpResult<$frame> {
                self.all_other(|e| e.mean())
            }

            pub fn median(&self) -> PhpResult<$frame> {
                self.all_other(|e| e.median())
            }

            pub fn min(&self) -> PhpResult<$frame> {
                self.all_other(|e| e.min())
            }

            pub fn max(&self) -> PhpResult<$frame> {
                self.all_other(|e| e.max())
            }

            pub fn first(&self) -> PhpResult<$frame> {
                self.all_other(|e| e.first())
            }

            pub fn last(&self) -> PhpResult<$frame> {
                self.all_other(|e| e.last())
            }

            pub fn count(&self) -> PhpResult<$frame> {
                self.all_other(|e| e.count())
            }

            pub fn nUnique(&self) -> PhpResult<$frame> {
                self.all_other(|e| e.n_unique())
            }

            /// Each group's values as lists.
            pub fn all(&self) -> PhpResult<$frame> {
                self.all_other(|e| e)
            }

            pub fn head(&self, n: Option<i64>) -> PhpResult<$frame> {
                $frame::out(self.grouped().head(Some(n.unwrap_or(5) as usize)))
            }

            pub fn tail(&self, n: Option<i64>) -> PhpResult<$frame> {
                $frame::out(self.grouped().tail(Some(n.unwrap_or(5) as usize)))
            }
        }
    };
}

fn pl_len() -> Expr {
    polars::prelude::len()
}

group_by!(PhpGroupBy, "Phpolars\\GroupBy", PhpDataFrame);
group_by!(PhpLazyGroupBy, "Phpolars\\LazyGroupBy", PhpLazyFrame);

/// concat([$a, $b], how: 'vertical'|'vertical_relaxed'|'horizontal'|'diagonal'|'diagonal_relaxed')
pub fn concat(items: &ZendHashTable, how: Option<String>) -> PhpResult<Zval> {
    use ext_php_rs::convert::IntoZval;
    let how = how.unwrap_or_else(|| "vertical".into());
    let values: Vec<&Zval> = items.values().collect();
    let first = values
        .first()
        .ok_or_else(|| value_error("concat() needs at least one item"))?;
    if first.extract::<&PhpSeries>().is_some() {
        let mut out = first.extract::<&PhpSeries>().unwrap().inner.clone();
        for v in &values[1..] {
            let s = v
                .extract::<&PhpSeries>()
                .ok_or_else(|| value_error("concat() items must all be the same kind"))?;
            out.append(&s.inner).php()?;
        }
        return Ok(PhpSeries { inner: out }.into_zval(false)?);
    }
    let eager = first.extract::<&PhpDataFrame>().is_some();
    let lfs = values
        .iter()
        .map(|v| other_frame(v))
        .collect::<PhpResult<Vec<_>>>()?;
    let args = UnionArgs {
        rechunk: true,
        to_supertypes: how.ends_with("_relaxed"),
        ..Default::default()
    };
    let lf = match how.as_str() {
        "vertical" | "vertical_relaxed" => polars::prelude::concat(lfs, args).php()?,
        "diagonal" | "diagonal_relaxed" => concat_lf_diagonal(lfs, args).php()?,
        "horizontal" => concat_lf_horizontal(lfs, HConcatOptions::default()).php()?,
        other => return Err(value_error(format!("unknown concat strategy '{other}'"))),
    };
    if eager {
        Ok(PhpDataFrame::out(lf)?.into_zval(false)?)
    } else {
        Ok(PhpLazyFrame::wrap(lf).into_zval(false)?)
    }
}

/// py describe(): count, null_count, mean, std, min, percentiles, and max.
/// Numeric and Boolean columns give floats. Other columns give strings:
/// temporal columns get a mean, min, max, and percentiles; strings get min
/// and max; nested columns only counts.
fn describe(df: &DataFrame, percentiles: Vec<f64>) -> PhpResult<PhpDataFrame> {
    let mut stats: Vec<String> = vec![
        "count".into(),
        "null_count".into(),
        "mean".into(),
        "std".into(),
        "min".into(),
    ];
    for p in &percentiles {
        stats.push(format!("{}%", p * 100.0));
    }
    stats.push("max".into());
    let none = || lit(Null {});
    let mut exprs = Vec::new();
    for c in df.columns() {
        let name = c.name().as_str();
        let dt = c.dtype();
        let numeric = dt.is_primitive_numeric();
        let boolean = dt.is_bool();
        let temporal = dt.is_temporal();
        let ordered = !dt.is_nested() && !dt.is_null();
        let x = || col(name);
        let mut row = vec![x().count(), x().null_count()];
        row.push(if numeric || boolean || temporal {
            x().mean()
        } else {
            none()
        });
        row.push(if numeric { x().std(1) } else { none() });
        row.push(if ordered { x().min() } else { none() });
        for p in &percentiles {
            row.push(match (numeric, temporal) {
                (true, _) => x().quantile(lit(*p), QuantileMethod::Nearest),
                (_, true) => x()
                    .quantile(lit(*p), QuantileMethod::Nearest)
                    .cast(dt.clone()),
                _ => none(),
            });
        }
        row.push(if ordered { x().max() } else { none() });
        for (i, e) in row.into_iter().enumerate() {
            let e = if numeric || boolean {
                e.cast(DataType::Float64)
            } else {
                // Trim a zero fraction so temporal means read like py's.
                e.cast(DataType::String)
                    .str()
                    .replace(lit("\\.0+$"), lit(""), false)
            };
            exprs.push(e.alias(format!("{name}\u{0}{i}")));
        }
    }
    init_threads();
    let one = df.clone().lazy().select(exprs).collect().php()?;
    let mut columns: Vec<Column> = vec![Series::new("statistic".into(), stats.clone()).into()];
    for c in df.columns() {
        let mut out: Option<Series> = None;
        for i in 0..stats.len() {
            let s = one
                .column(&format!("{}\u{0}{i}", c.name()))
                .php()?
                .as_materialized_series()
                .clone();
            match out.as_mut() {
                None => out = Some(s),
                Some(o) => {
                    o.append(&s).php()?;
                }
            }
        }
        columns.push(out.unwrap().with_name(c.name().clone()).into());
    }
    DataFrame::new_infer_height(columns)
        .php()
        .map(PhpDataFrame::wrap)
}

/// Convenience for exprs that take an IntoExpr argument.
#[allow(dead_code)]
fn expr_arg(v: &Zval) -> PhpResult<Expr> {
    into_expr(v)
}
