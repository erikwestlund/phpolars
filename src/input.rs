//! Reading PHP arguments the way py-polars reads Python ones.
use ext_php_rs::{prelude::*, types::Zval};
use polars::prelude::*;

use crate::{
    convert::{is_list, literal},
    error::value_error,
    expr::{PhpExpr, PhpThen},
    series::PhpSeries,
};

/// Omitted named arguments and explicit nulls both mean "not given".
pub fn given(v: Option<&Zval>) -> Option<&Zval> {
    v.filter(|v| !v.is_null())
}

/// py "IntoExpr": strings name columns, Expr passes through, anything else is a literal.
pub fn into_expr(v: &Zval) -> PhpResult<Expr> {
    if let Some(s) = v.str() {
        return Ok(col(s));
    }
    if let Some(e) = v.extract::<&PhpExpr>() {
        return Ok(e.inner.clone());
    }
    if let Some(t) = v.extract::<&PhpThen>() {
        return Ok(t.expr());
    }
    if let Some(s) = v.extract::<&PhpSeries>() {
        return Ok(lit(s.inner.clone()));
    }
    literal(v)
}

/// A comparison or arithmetic operand: strings are literals, Expr passes through.
pub fn operand(v: &Zval) -> PhpResult<Expr> {
    if let Some(t) = v.extract::<&PhpThen>() {
        return Ok(t.expr());
    }
    literal(v)
}

/// The right-hand side of isIn(): a PHP list or Series is one collection, as in py.
pub fn collection(v: &Zval) -> PhpResult<Expr> {
    if v.array().is_some() || v.extract::<&PhpSeries>().is_some() {
        return Ok(literal(v)?.implode(true));
    }
    operand(v)
}

/// py *exprs and **named: each argument is a column name, an Expr, a list of
/// those, or ['alias' => Expr|name].
pub fn exprs(args: &[&Zval]) -> PhpResult<Vec<Expr>> {
    let mut out = Vec::with_capacity(args.len());
    for arg in args {
        push_exprs(arg, &mut out)?;
    }
    Ok(out)
}

fn push_exprs(v: &Zval, out: &mut Vec<Expr>) -> PhpResult<()> {
    match v.array() {
        Some(ht) if is_list(ht) => {
            for item in ht.values() {
                push_exprs(item, out)?;
            }
        }
        Some(ht) => {
            for (k, item) in ht.iter() {
                out.push(into_expr(item)?.alias(k.to_string()));
            }
        }
        None => out.push(into_expr(v)?),
    }
    Ok(())
}

/// Column names given as strings or lists of strings.
pub fn names(args: &[&Zval]) -> PhpResult<Vec<String>> {
    let mut out = Vec::new();
    for arg in args {
        if let Some(s) = arg.str() {
            out.push(s.to_owned());
        } else if let Some(ht) = arg.array() {
            for item in ht.values() {
                out.push(
                    item.str()
                        .ok_or_else(|| value_error("expected column names"))?
                        .to_owned(),
                );
            }
        } else {
            return Err(value_error("expected column names"));
        }
    }
    Ok(out)
}

pub fn opt_names(v: Option<&Zval>) -> PhpResult<Option<Vec<String>>> {
    match v {
        None => Ok(None),
        Some(v) if v.is_null() => Ok(None),
        Some(v) => names(&[v]).map(Some),
    }
}

/// A bool or one bool per key, as py descending=/nulls_last= take.
pub fn bools(v: Option<&Zval>, default: bool) -> PhpResult<Vec<bool>> {
    match v {
        None => Ok(vec![default]),
        Some(v) if v.is_null() => Ok(vec![default]),
        Some(v) => {
            if let Some(b) = v.bool() {
                Ok(vec![b])
            } else if let Some(ht) = v.array() {
                ht.values()
                    .map(|b| b.bool().ok_or_else(|| value_error("expected a bool or list of bools")))
                    .collect()
            } else {
                Err(value_error("expected a bool or list of bools"))
            }
        }
    }
}

pub fn closed_interval(s: Option<String>) -> PhpResult<ClosedInterval> {
    match s.as_deref().unwrap_or("both") {
        "both" => Ok(ClosedInterval::Both),
        "left" => Ok(ClosedInterval::Left),
        "right" => Ok(ClosedInterval::Right),
        "none" => Ok(ClosedInterval::None),
        other => Err(value_error(format!("closed must be both, left, right or none, not '{other}'"))),
    }
}

pub fn time_unit(s: Option<String>) -> PhpResult<TimeUnit> {
    match s.as_deref().unwrap_or("us") {
        "ns" => Ok(TimeUnit::Nanoseconds),
        "us" => Ok(TimeUnit::Microseconds),
        "ms" => Ok(TimeUnit::Milliseconds),
        other => Err(value_error(format!("time unit must be ns, us or ms, not '{other}'"))),
    }
}
