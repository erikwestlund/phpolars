#![allow(non_snake_case)]
//! Phpolars\Expr: py-polars Expr with camelCase method names. Expressions
//! never hold PHP closures, so they can run on Polars worker threads.
use ext_php_rs::{prelude::*, types::Zval};
use polars::prelude::{self as pl, *};

use crate::{
    dtype,
    error::{OrPhp, value_error},
    input::{bools, closed_interval, exprs, into_expr, operand},
    namespace::{PhpDtNameSpace, PhpListNameSpace, PhpNameNameSpace, PhpStrNameSpace, PhpStructNameSpace},
};

#[php_class]
#[php(name = "Phpolars\\Expr")]
#[derive(Clone)]
pub struct PhpExpr {
    pub(crate) inner: Expr,
}

pub fn wrap(inner: Expr) -> PhpExpr {
    PhpExpr { inner }
}

pub fn quantile_method(s: Option<String>) -> PhpResult<QuantileMethod> {
    Ok(match s.as_deref().unwrap_or("nearest") {
        "nearest" => QuantileMethod::Nearest,
        "lower" => QuantileMethod::Lower,
        "higher" => QuantileMethod::Higher,
        "midpoint" => QuantileMethod::Midpoint,
        "linear" => QuantileMethod::Linear,
        "equiprobable" => QuantileMethod::Equiprobable,
        other => return Err(value_error(format!("unknown interpolation '{other}'"))),
    })
}

pub fn rank_method(s: Option<String>) -> PhpResult<RankMethod> {
    Ok(match s.as_deref().unwrap_or("average") {
        "average" => RankMethod::Average,
        "min" => RankMethod::Min,
        "max" => RankMethod::Max,
        "dense" => RankMethod::Dense,
        "ordinal" => RankMethod::Ordinal,
        "random" => RankMethod::Random,
        other => return Err(value_error(format!("unknown rank method '{other}'"))),
    })
}

fn fill_strategy(s: &str, limit: Option<i64>) -> PhpResult<FillNullStrategy> {
    let limit = limit.map(|l| l as IdxSize);
    Ok(match s {
        "forward" => FillNullStrategy::Forward(limit),
        "backward" => FillNullStrategy::Backward(limit),
        "mean" => FillNullStrategy::Mean,
        "min" => FillNullStrategy::Min,
        "max" => FillNullStrategy::Max,
        "zero" => FillNullStrategy::Zero,
        "one" => FillNullStrategy::One,
        other => return Err(value_error(format!("unknown fill strategy '{other}'"))),
    })
}

fn rolling(windowSize: i64, minPeriods: Option<i64>, center: Option<bool>) -> RollingOptionsFixedWindow {
    RollingOptionsFixedWindow {
        window_size: windowSize as usize,
        min_periods: minPeriods.unwrap_or(windowSize) as usize,
        center: center.unwrap_or(false),
        ..Default::default()
    }
}

pub fn sample(e: Expr, n: Option<i64>, fraction: Option<crate::input::Num>, withReplacement: Option<bool>, shuffle: Option<bool>, seed: Option<i64>) -> PhpResult<Expr> {
    let wr = withReplacement.unwrap_or(false);
    let sh = shuffle;
    let seed = seed.map(|s| s as u64);
    Ok(match (n, fraction) {
        (Some(_), Some(_)) => return Err(value_error("sample() takes n or fraction, not both")),
        (_, Some(f)) => e.sample_frac(lit(f.0), wr, sh, seed),
        (n, None) => e.sample_n(lit(n.unwrap_or(1) as u64), wr, sh, seed),
    })
}

pub fn clip(e: Expr, lowerBound: Option<&Zval>, upperBound: Option<&Zval>) -> PhpResult<Expr> {
    let lo = lowerBound.filter(|v| !v.is_null()).map(operand).transpose()?;
    let hi = upperBound.filter(|v| !v.is_null()).map(operand).transpose()?;
    Ok(match (lo, hi) {
        (Some(lo), Some(hi)) => e.clip(lo, hi),
        (Some(lo), None) => e.clip_min(lo),
        (None, Some(hi)) => e.clip_max(hi),
        (None, None) => e,
    })
}

/// Fill null with a value, or with a strategy; shared by Expr and Series.
pub fn fill_null(e: Expr, value: Option<&Zval>, strategy: Option<String>, limit: Option<i64>) -> PhpResult<Expr> {
    match (value, strategy) {
        (Some(v), None) if !v.is_null() => Ok(e.fill_null(operand(v)?)),
        (_, Some(s)) => Ok(e.fill_null_with_strategy(fill_strategy(&s, limit)?)),
        _ => Err(value_error("fillNull() needs a value or a strategy")),
    }
}

#[php_impl]
impl PhpExpr {
    // Arithmetic. Operands may be an Expr or a literal; strings are literals.

    pub fn add(&self, other: &Zval) -> PhpResult<Self> {
        Ok(wrap(self.inner.clone() + operand(other)?))
    }

    pub fn sub(&self, other: &Zval) -> PhpResult<Self> {
        Ok(wrap(self.inner.clone() - operand(other)?))
    }

    pub fn mul(&self, other: &Zval) -> PhpResult<Self> {
        Ok(wrap(self.inner.clone() * operand(other)?))
    }

    /// True division, always floating point, as py's /.
    pub fn truediv(&self, other: &Zval) -> PhpResult<Self> {
        Ok(wrap(self.inner.clone().true_div(operand(other)?)))
    }

    pub fn div(&self, other: &Zval) -> PhpResult<Self> {
        self.truediv(other)
    }

    pub fn floordiv(&self, other: &Zval) -> PhpResult<Self> {
        Ok(wrap(self.inner.clone().floor_div(operand(other)?)))
    }

    #[php(name = "mod")]
    pub fn modulo(&self, other: &Zval) -> PhpResult<Self> {
        Ok(wrap(self.inner.clone() % operand(other)?))
    }

    pub fn pow(&self, exponent: &Zval) -> PhpResult<Self> {
        Ok(wrap(self.inner.clone().pow(operand(exponent)?)))
    }

    pub fn neg(&self) -> Self {
        wrap(lit(0) - self.inner.clone())
    }

    // Comparison.

    pub fn eq(&self, other: &Zval) -> PhpResult<Self> {
        Ok(wrap(self.inner.clone().eq(operand(other)?)))
    }

    pub fn eqMissing(&self, other: &Zval) -> PhpResult<Self> {
        Ok(wrap(self.inner.clone().eq_missing(operand(other)?)))
    }

    pub fn ne(&self, other: &Zval) -> PhpResult<Self> {
        Ok(wrap(self.inner.clone().neq(operand(other)?)))
    }

    pub fn neMissing(&self, other: &Zval) -> PhpResult<Self> {
        Ok(wrap(self.inner.clone().neq_missing(operand(other)?)))
    }

    pub fn gt(&self, other: &Zval) -> PhpResult<Self> {
        Ok(wrap(self.inner.clone().gt(operand(other)?)))
    }

    pub fn ge(&self, other: &Zval) -> PhpResult<Self> {
        Ok(wrap(self.inner.clone().gt_eq(operand(other)?)))
    }

    pub fn lt(&self, other: &Zval) -> PhpResult<Self> {
        Ok(wrap(self.inner.clone().lt(operand(other)?)))
    }

    pub fn le(&self, other: &Zval) -> PhpResult<Self> {
        Ok(wrap(self.inner.clone().lt_eq(operand(other)?)))
    }

    pub fn isBetween(&self, lower: &Zval, upper: &Zval, closed: Option<String>) -> PhpResult<Self> {
        Ok(wrap(self.inner.clone().is_between(operand(lower)?, operand(upper)?, closed_interval(closed)?)))
    }

    /// Membership in a list, Series or list-valued Expr.
    pub fn isIn(&self, other: &Zval, nullsEqual: Option<bool>) -> PhpResult<Self> {
        Ok(wrap(self.inner.clone().is_in(crate::input::collection(other)?, nullsEqual.unwrap_or(false))))
    }

    // Boolean.

    pub fn and(&self, other: &Zval) -> PhpResult<Self> {
        Ok(wrap(self.inner.clone().and(operand(other)?)))
    }

    pub fn or(&self, other: &Zval) -> PhpResult<Self> {
        Ok(wrap(self.inner.clone().or(operand(other)?)))
    }

    pub fn xor(&self, other: &Zval) -> PhpResult<Self> {
        Ok(wrap(self.inner.clone().xor(operand(other)?)))
    }

    pub fn not(&self) -> Self {
        wrap(self.inner.clone().not())
    }

    pub fn any(&self, ignoreNulls: Option<bool>) -> Self {
        wrap(self.inner.clone().any(ignoreNulls.unwrap_or(true)))
    }

    pub fn all(&self, ignoreNulls: Option<bool>) -> Self {
        wrap(self.inner.clone().all(ignoreNulls.unwrap_or(true)))
    }

    // Missing values.

    pub fn isNull(&self) -> Self {
        wrap(self.inner.clone().is_null())
    }

    pub fn isNotNull(&self) -> Self {
        wrap(self.inner.clone().is_not_null())
    }

    pub fn isNan(&self) -> Self {
        wrap(self.inner.clone().is_nan())
    }

    pub fn isNotNan(&self) -> Self {
        wrap(self.inner.clone().is_not_nan())
    }

    pub fn isFinite(&self) -> Self {
        wrap(self.inner.clone().is_finite())
    }

    pub fn isInfinite(&self) -> Self {
        wrap(self.inner.clone().is_infinite())
    }

    /// fillNull(value) or fillNull(strategy: 'forward'|'backward'|'mean'|'min'|'max'|'zero'|'one').
    pub fn fillNull(&self, value: Option<&Zval>, strategy: Option<String>, limit: Option<i64>) -> PhpResult<Self> {
        fill_null(self.inner.clone(), value, strategy, limit).map(wrap)
    }

    pub fn fillNan(&self, value: &Zval) -> PhpResult<Self> {
        Ok(wrap(self.inner.clone().fill_nan(operand(value)?)))
    }

    pub fn forwardFill(&self, limit: Option<i64>) -> PhpResult<Self> {
        Ok(wrap(self.inner.clone().fill_null_with_strategy(fill_strategy("forward", limit)?)))
    }

    pub fn backwardFill(&self, limit: Option<i64>) -> PhpResult<Self> {
        Ok(wrap(self.inner.clone().fill_null_with_strategy(fill_strategy("backward", limit)?)))
    }

    pub fn dropNulls(&self) -> Self {
        wrap(self.inner.clone().drop_nulls())
    }

    pub fn dropNans(&self) -> Self {
        wrap(self.inner.clone().drop_nans())
    }

    pub fn nullCount(&self) -> Self {
        wrap(self.inner.clone().null_count())
    }

    pub fn hasNulls(&self) -> Self {
        wrap(self.inner.clone().has_nulls())
    }

    pub fn interpolate(&self, method: Option<String>) -> PhpResult<Self> {
        let m = match method.as_deref().unwrap_or("linear") {
            "linear" => InterpolationMethod::Linear,
            "nearest" => InterpolationMethod::Nearest,
            other => return Err(value_error(format!("unknown interpolation '{other}'"))),
        };
        Ok(wrap(self.inner.clone().interpolate(m)))
    }

    // Types and names.

    pub fn cast(&self, dtype: String, strict: Option<bool>) -> PhpResult<Self> {
        let dt = dtype::parse(&dtype)?;
        Ok(wrap(if strict.unwrap_or(true) {
            self.inner.clone().strict_cast(dt)
        } else {
            self.inner.clone().cast(dt)
        }))
    }

    pub fn alias(&self, name: String) -> Self {
        wrap(self.inner.clone().alias(name))
    }

    pub fn name(&self) -> PhpNameNameSpace {
        PhpNameNameSpace::new(self.inner.clone())
    }

    pub fn str(&self) -> PhpStrNameSpace {
        PhpStrNameSpace::new(self.inner.clone(), None)
    }

    pub fn dt(&self) -> PhpDtNameSpace {
        PhpDtNameSpace::new(self.inner.clone(), None)
    }

    pub fn list(&self) -> PhpListNameSpace {
        PhpListNameSpace::new(self.inner.clone(), None)
    }

    #[php(name = "struct")]
    pub fn struct_ns(&self) -> PhpStructNameSpace {
        PhpStructNameSpace::new(self.inner.clone(), None)
    }

    // Aggregations.

    pub fn sum(&self) -> Self {
        wrap(self.inner.clone().sum())
    }

    pub fn mean(&self) -> Self {
        wrap(self.inner.clone().mean())
    }

    pub fn median(&self) -> Self {
        wrap(self.inner.clone().median())
    }

    pub fn min(&self) -> Self {
        wrap(self.inner.clone().min())
    }

    pub fn max(&self) -> Self {
        wrap(self.inner.clone().max())
    }

    pub fn nanMin(&self) -> Self {
        wrap(self.inner.clone().nan_min())
    }

    pub fn nanMax(&self) -> Self {
        wrap(self.inner.clone().nan_max())
    }

    pub fn std(&self, ddof: Option<i64>) -> Self {
        wrap(self.inner.clone().std(ddof.unwrap_or(1) as u8))
    }

    pub fn var(&self, ddof: Option<i64>) -> Self {
        wrap(self.inner.clone().var(ddof.unwrap_or(1) as u8))
    }

    pub fn quantile(&self, quantile: &Zval, interpolation: Option<String>) -> PhpResult<Self> {
        Ok(wrap(self.inner.clone().quantile(operand(quantile)?, quantile_method(interpolation)?)))
    }

    pub fn product(&self) -> Self {
        wrap(self.inner.clone().product())
    }

    /// Non-null values.
    pub fn count(&self) -> Self {
        wrap(self.inner.clone().count())
    }

    /// All values, nulls included.
    pub fn len(&self) -> Self {
        wrap(self.inner.clone().len())
    }

    pub fn nUnique(&self) -> Self {
        wrap(self.inner.clone().n_unique())
    }

    pub fn approxNUnique(&self) -> Self {
        wrap(self.inner.clone().approx_n_unique())
    }

    pub fn first(&self) -> Self {
        wrap(self.inner.clone().first())
    }

    pub fn last(&self) -> Self {
        wrap(self.inner.clone().last())
    }

    pub fn mode(&self) -> Self {
        wrap(self.inner.clone().mode(true))
    }

    pub fn argMin(&self) -> Self {
        wrap(self.inner.clone().arg_min())
    }

    pub fn argMax(&self) -> Self {
        wrap(self.inner.clone().arg_max())
    }

    pub fn skew(&self, bias: Option<bool>) -> Self {
        wrap(self.inner.clone().skew(bias.unwrap_or(true)))
    }

    pub fn kurtosis(&self, fisher: Option<bool>, bias: Option<bool>) -> Self {
        wrap(self.inner.clone().kurtosis(fisher.unwrap_or(true), bias.unwrap_or(true)))
    }

    pub fn implode(&self) -> Self {
        wrap(self.inner.clone().implode(true))
    }

    // Windows and order-dependent operations.

    /// over('g', 'h', orderBy: 'x'): a window over partitions.
    pub fn over(&self, partitionBy: &Zval, orderBy: Option<&Zval>, descending: Option<bool>, nullsLast: Option<bool>, mappingStrategy: Option<String>) -> PhpResult<Self> {
        let partition = if partitionBy.is_null() { vec![] } else { exprs(&[partitionBy])? };
        let order = match orderBy {
            Some(o) if !o.is_null() => Some((
                exprs(&[o])?,
                SortOptions::default()
                    .with_order_descending(descending.unwrap_or(false))
                    .with_nulls_last(nullsLast.unwrap_or(false)),
            )),
            _ => None,
        };
        let mapping = match mappingStrategy.as_deref().unwrap_or("group_to_rows") {
            "group_to_rows" => WindowMapping::GroupsToRows,
            "explode" => WindowMapping::Explode,
            "join" => WindowMapping::Join,
            other => return Err(value_error(format!("unknown mapping strategy '{other}'"))),
        };
        let partition = if partition.is_empty() { None } else { Some(partition) };
        self.inner.clone().over_with_options(partition, order, mapping).php().map(wrap)
    }

    pub fn shift(&self, n: Option<i64>, fillValue: Option<&Zval>) -> PhpResult<Self> {
        let n = lit(n.unwrap_or(1));
        Ok(wrap(match fillValue {
            Some(v) if !v.is_null() => self.inner.clone().shift_and_fill(n, operand(v)?),
            _ => self.inner.clone().shift(n),
        }))
    }

    pub fn diff(&self, n: Option<i64>, nullBehavior: Option<String>) -> PhpResult<Self> {
        let nb = match nullBehavior.as_deref().unwrap_or("ignore") {
            "ignore" => polars_core::series::ops::NullBehavior::Ignore,
            "drop" => polars_core::series::ops::NullBehavior::Drop,
            other => return Err(value_error(format!("unknown null behavior '{other}'"))),
        };
        Ok(wrap(self.inner.clone().diff(lit(n.unwrap_or(1)), nb)))
    }

    pub fn pctChange(&self, n: Option<i64>) -> Self {
        wrap(self.inner.clone().pct_change(lit(n.unwrap_or(1))))
    }

    pub fn cumSum(&self, reverse: Option<bool>) -> Self {
        wrap(self.inner.clone().cum_sum(reverse.unwrap_or(false)))
    }

    pub fn cumProd(&self, reverse: Option<bool>) -> Self {
        wrap(self.inner.clone().cum_prod(reverse.unwrap_or(false)))
    }

    pub fn cumMin(&self, reverse: Option<bool>) -> Self {
        wrap(self.inner.clone().cum_min(reverse.unwrap_or(false)))
    }

    pub fn cumMax(&self, reverse: Option<bool>) -> Self {
        wrap(self.inner.clone().cum_max(reverse.unwrap_or(false)))
    }

    pub fn cumCount(&self, reverse: Option<bool>) -> Self {
        wrap(self.inner.clone().cum_count(reverse.unwrap_or(false)))
    }

    pub fn rank(&self, method: Option<String>, descending: Option<bool>, seed: Option<i64>) -> PhpResult<Self> {
        let options = RankOptions {
            method: rank_method(method)?,
            descending: descending.unwrap_or(false),
        };
        Ok(wrap(self.inner.clone().rank(options, seed.map(|s| s as u64))))
    }

    pub fn rollingMean(&self, windowSize: i64, minPeriods: Option<i64>, center: Option<bool>) -> Self {
        wrap(self.inner.clone().rolling_mean(rolling(windowSize, minPeriods, center)))
    }

    pub fn rollingSum(&self, windowSize: i64, minPeriods: Option<i64>, center: Option<bool>) -> Self {
        wrap(self.inner.clone().rolling_sum(rolling(windowSize, minPeriods, center)))
    }

    pub fn rollingMin(&self, windowSize: i64, minPeriods: Option<i64>, center: Option<bool>) -> Self {
        wrap(self.inner.clone().rolling_min(rolling(windowSize, minPeriods, center)))
    }

    pub fn rollingMax(&self, windowSize: i64, minPeriods: Option<i64>, center: Option<bool>) -> Self {
        wrap(self.inner.clone().rolling_max(rolling(windowSize, minPeriods, center)))
    }

    pub fn rollingMedian(&self, windowSize: i64, minPeriods: Option<i64>, center: Option<bool>) -> Self {
        wrap(self.inner.clone().rolling_median(rolling(windowSize, minPeriods, center)))
    }

    pub fn rollingStd(&self, windowSize: i64, minPeriods: Option<i64>, center: Option<bool>) -> Self {
        wrap(self.inner.clone().rolling_std(rolling(windowSize, minPeriods, center)))
    }

    pub fn rollingVar(&self, windowSize: i64, minPeriods: Option<i64>, center: Option<bool>) -> Self {
        wrap(self.inner.clone().rolling_var(rolling(windowSize, minPeriods, center)))
    }

    // Sorting, selecting and reshaping.

    pub fn sort(&self, descending: Option<bool>, nullsLast: Option<bool>) -> Self {
        wrap(self.inner.clone().sort(
            SortOptions::default()
                .with_order_descending(descending.unwrap_or(false))
                .with_nulls_last(nullsLast.unwrap_or(false))
                .with_maintain_order(true),
        ))
    }

    pub fn sortBy(&self, by: &Zval, descending: Option<&Zval>, nullsLast: Option<&Zval>) -> PhpResult<Self> {
        let by = exprs(&[by])?;
        let options = SortMultipleOptions::default()
            .with_order_descending_multi(bools(descending, false)?)
            .with_nulls_last_multi(bools(nullsLast, false)?)
            .with_maintain_order(true);
        Ok(wrap(self.inner.clone().sort_by(by, options)))
    }

    pub fn argSort(&self, descending: Option<bool>, nullsLast: Option<bool>) -> Self {
        wrap(self.inner.clone().arg_sort(descending.unwrap_or(false), nullsLast.unwrap_or(false)))
    }

    pub fn reverse(&self) -> Self {
        wrap(self.inner.clone().reverse())
    }

    pub fn head(&self, n: Option<i64>) -> Self {
        wrap(self.inner.clone().head(Some(n.unwrap_or(10) as usize)))
    }

    pub fn tail(&self, n: Option<i64>) -> Self {
        wrap(self.inner.clone().tail(Some(n.unwrap_or(10) as usize)))
    }

    pub fn slice(&self, offset: i64, length: Option<i64>) -> Self {
        let len = match length {
            Some(l) => lit(l as u64),
            None => lit(IdxSize::MAX as u64),
        };
        wrap(self.inner.clone().slice(lit(offset), len))
    }

    pub fn topK(&self, k: Option<i64>) -> Self {
        wrap(self.inner.clone().top_k(lit(k.unwrap_or(5) as u64)))
    }

    pub fn bottomK(&self, k: Option<i64>) -> Self {
        wrap(self.inner.clone().bottom_k(lit(k.unwrap_or(5) as u64)))
    }

    pub fn filter(&self, predicate: &Zval) -> PhpResult<Self> {
        Ok(wrap(self.inner.clone().filter(into_expr(predicate)?)))
    }

    pub fn gather(&self, indices: &Zval) -> PhpResult<Self> {
        Ok(wrap(self.inner.clone().gather(operand(indices)?, false)))
    }

    /// The value at a position; negative positions count from the end.
    pub fn get(&self, index: &Zval, nullOnOob: Option<bool>) -> PhpResult<Self> {
        Ok(wrap(self.inner.clone().get(operand(index)?, nullOnOob.unwrap_or(false))))
    }

    pub fn gatherEvery(&self, n: i64, offset: Option<i64>) -> Self {
        wrap(self.inner.clone().gather_every(n as usize, offset.unwrap_or(0) as usize))
    }

    pub fn explode(&self) -> Self {
        wrap(self.inner.clone().explode(ExplodeOptions { empty_as_null: true, keep_nulls: true }))
    }

    pub fn unique(&self, maintainOrder: Option<bool>) -> Self {
        wrap(if maintainOrder.unwrap_or(false) {
            self.inner.clone().unique_stable()
        } else {
            self.inner.clone().unique()
        })
    }

    pub fn uniqueCounts(&self) -> Self {
        wrap(self.inner.clone().unique_counts())
    }

    pub fn isUnique(&self) -> Self {
        wrap(self.inner.clone().is_unique())
    }

    pub fn isDuplicated(&self) -> Self {
        wrap(self.inner.clone().is_duplicated())
    }

    pub fn isFirstDistinct(&self) -> Self {
        wrap(self.inner.clone().is_first_distinct())
    }

    pub fn isLastDistinct(&self) -> Self {
        wrap(self.inner.clone().is_last_distinct())
    }

    pub fn valueCounts(&self, sort: Option<bool>, name: Option<String>, normalize: Option<bool>) -> Self {
        let normalize = normalize.unwrap_or(false);
        let name = name.unwrap_or_else(|| if normalize { "proportion".into() } else { "count".into() });
        wrap(self.inner.clone().value_counts(sort.unwrap_or(false), true, &name, normalize))
    }

    pub fn shuffle(&self, seed: Option<i64>) -> Self {
        wrap(self.inner.clone().shuffle(seed.map(|s| s as u64)))
    }

    pub fn sample(&self, n: Option<i64>, fraction: Option<crate::input::Num>, withReplacement: Option<bool>, shuffle: Option<bool>, seed: Option<i64>) -> PhpResult<Self> {
        sample(self.inner.clone(), n, fraction, withReplacement, shuffle, seed).map(wrap)
    }

    pub fn repeatBy(&self, by: &Zval) -> PhpResult<Self> {
        Ok(wrap(self.inner.clone().repeat_by(operand(by)?)))
    }

    pub fn append(&self, other: &Zval, upcast: Option<bool>) -> PhpResult<Self> {
        Ok(wrap(self.inner.clone().append(into_expr(other)?, upcast.unwrap_or(true))))
    }

    /// replace(old, new) or replace([old => new, ...]).
    pub fn replace(&self, old: &Zval, new: Option<&Zval>) -> PhpResult<Self> {
        let (o, n) = replace_args(old, new)?;
        Ok(wrap(self.inner.clone().replace(o, n)))
    }

    pub fn replaceStrict(&self, old: &Zval, new: Option<&Zval>, default: Option<&Zval>, returnDtype: Option<String>) -> PhpResult<Self> {
        let (o, n) = replace_args(old, new)?;
        let default = match default {
            Some(d) if !d.is_null() => Some(operand(d)?),
            _ => None,
        };
        let dt = returnDtype.map(|d| dtype::parse(&d)).transpose()?;
        Ok(wrap(self.inner.clone().replace_strict(o, n, default, dt.map(DataTypeExpr::from))))
    }

    pub fn clip(&self, lowerBound: Option<&Zval>, upperBound: Option<&Zval>) -> PhpResult<Self> {
        clip(self.inner.clone(), lowerBound, upperBound).map(wrap)
    }

    pub fn dot(&self, other: &Zval) -> PhpResult<Self> {
        Ok(wrap(self.inner.clone().dot(into_expr(other)?)))
    }

    // Math.

    pub fn abs(&self) -> Self {
        wrap(self.inner.clone().abs())
    }

    pub fn round(&self, decimals: Option<i64>, mode: Option<String>) -> PhpResult<Self> {
        let mode = match mode.as_deref().unwrap_or("half_to_even") {
            "half_to_even" => RoundMode::HalfToEven,
            "half_away_from_zero" => RoundMode::HalfAwayFromZero,
            other => return Err(value_error(format!("unknown round mode '{other}'"))),
        };
        Ok(wrap(self.inner.clone().round(decimals.unwrap_or(0) as u32, mode)))
    }

    pub fn roundSigFigs(&self, digits: i64) -> Self {
        wrap(self.inner.clone().round_sig_figs(digits as i32))
    }

    pub fn floor(&self) -> Self {
        wrap(self.inner.clone().floor())
    }

    pub fn ceil(&self) -> Self {
        wrap(self.inner.clone().ceil())
    }

    pub fn sqrt(&self) -> Self {
        wrap(self.inner.clone().sqrt())
    }

    pub fn cbrt(&self) -> Self {
        wrap(self.inner.clone().cbrt())
    }

    pub fn exp(&self) -> Self {
        wrap(self.inner.clone().exp())
    }

    pub fn log(&self, base: Option<crate::input::Num>) -> Self {
        wrap(self.inner.clone().log(lit(base.map_or(std::f64::consts::E, |b| b.0))))
    }

    pub fn log10(&self) -> Self {
        wrap(self.inner.clone().log(lit(10.0)))
    }

    pub fn log1p(&self) -> Self {
        wrap(self.inner.clone().log1p())
    }

    pub fn sign(&self) -> Self {
        wrap(self.inner.clone().sign())
    }

    pub fn sin(&self) -> Self {
        wrap(self.inner.clone().sin())
    }

    pub fn cos(&self) -> Self {
        wrap(self.inner.clone().cos())
    }

    pub fn tan(&self) -> Self {
        wrap(self.inner.clone().tan())
    }

    pub fn arcsin(&self) -> Self {
        wrap(self.inner.clone().arcsin())
    }

    pub fn arccos(&self) -> Self {
        wrap(self.inner.clone().arccos())
    }

    pub fn arctan(&self) -> Self {
        wrap(self.inner.clone().arctan())
    }

    pub fn __to_string(&self) -> String {
        format!("{:?}", self.inner)
    }
}

/// replace(old, new) or replace([old => new]).
fn replace_args(old: &Zval, new: Option<&Zval>) -> PhpResult<(Expr, Expr)> {
    match crate::input::given(new) {
        Some(n) => Ok((operand(old)?, operand(n)?)),
        None => {
            let map = old
                .array()
                .ok_or_else(|| value_error("replace() takes (old, new) or a mapping array"))?;
            let mut olds = Vec::new();
            let mut news = Vec::new();
            for (k, v) in map.iter() {
                olds.push(match k {
                    ext_php_rs::types::ArrayKey::Long(i) => AnyValue::Int64(i),
                    k => AnyValue::StringOwned(k.to_string().into()),
                });
                news.push(crate::convert::any_value(v)?);
            }
            let o = Series::from_any_values("old".into(), &olds, false).php()?;
            let n = Series::from_any_values("new".into(), &news, false).php()?;
            Ok((lit(o), lit(n)))
        }
    }
}

/// pl.when(cond)->then(value)[->when(cond)->then(value)...]->otherwise(value)
#[php_class]
#[php(name = "Phpolars\\When")]
pub struct PhpWhen {
    branches: Vec<(Expr, Expr)>,
    condition: Expr,
}

#[php_class]
#[php(name = "Phpolars\\Then")]
pub struct PhpThen {
    branches: Vec<(Expr, Expr)>,
}

impl PhpThen {
    fn build(&self, otherwise: Expr) -> Expr {
        let mut it = self.branches.iter().cloned();
        let (c, v) = it.next().expect("then() always records a branch");
        let then = pl::when(c).then(v);
        let mut rest = it.peekable();
        if rest.peek().is_none() {
            return then.otherwise(otherwise);
        }
        let (c, v) = rest.next().unwrap();
        let mut chained = then.when(c).then(v);
        for (c, v) in rest {
            chained = chained.when(c).then(v);
        }
        chained.otherwise(otherwise)
    }

    /// The chain with an implicit otherwise(null), as py treats a bare Then.
    pub fn expr(&self) -> Expr {
        self.build(lit(Null {}))
    }
}

pub fn when_start(condition: &[&Zval]) -> PhpResult<PhpWhen> {
    Ok(PhpWhen {
        branches: Vec::new(),
        condition: all_of(condition)?,
    })
}

/// Several when() conditions are combined with and, as in py.
fn all_of(conditions: &[&Zval]) -> PhpResult<Expr> {
    let mut es = exprs(conditions)?.into_iter();
    let first = es.next().ok_or_else(|| value_error("when() needs a condition"))?;
    Ok(es.fold(first, |acc, e| acc.and(e)))
}

#[php_impl]
impl PhpWhen {
    /// Strings name columns, as in py; use lit() for string values.
    pub fn then(&self, value: &Zval) -> PhpResult<PhpThen> {
        let mut branches = self.branches.clone();
        branches.push((self.condition.clone(), into_expr(value)?));
        Ok(PhpThen { branches })
    }
}

#[php_impl]
impl PhpThen {
    pub fn when(&self, condition: &[&Zval]) -> PhpResult<PhpWhen> {
        Ok(PhpWhen {
            branches: self.branches.clone(),
            condition: all_of(condition)?,
        })
    }

    pub fn otherwise(&self, value: &Zval) -> PhpResult<PhpExpr> {
        Ok(wrap(self.build(into_expr(value)?)))
    }

    pub fn alias(&self, name: String) -> PhpExpr {
        wrap(self.expr().alias(name))
    }
}
