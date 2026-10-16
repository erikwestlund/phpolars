//! Reading PHP arguments the way py-polars reads Python ones.
use ext_php_rs::{convert::FromZval, flags::DataType as PhpType, prelude::*, types::Zval};
use polars::prelude::*;

use crate::{
    convert::{is_list, literal},
    error::value_error,
    expr::{PhpExpr, PhpThen},
    series::PhpSeries,
};

/// A PHP int or float argument. ext-php-rs does not coerce int to f64 the
/// way PHP's own parameter parsing does, so float parameters use this.
#[derive(Clone, Copy, Debug)]
pub struct Num(pub f64);

impl<'a> FromZval<'a> for Num {
    const TYPE: PhpType = PhpType::Double;

    fn from_zval(zval: &'a Zval) -> Option<Self> {
        zval.double()
            .or_else(|| zval.long().map(|i| i as f64))
            .map(Num)
    }
}

/// A count, length, or position argument: negative values are an error
/// rather than wrapping to a huge unsigned number.
pub fn count(v: i64, what: &str) -> PhpResult<usize> {
    usize::try_from(v).map_err(|_| value_error(format!("{what} must not be negative, got {v}")))
}

/// A degrees-of-freedom argument (default 1), as Polars' u8.
pub fn ddof(v: Option<i64>) -> PhpResult<u8> {
    let d = v.unwrap_or(1);
    u8::try_from(d).map_err(|_| value_error(format!("ddof must be between 0 and 255, got {d}")))
}

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
                    .map(|b| {
                        b.bool()
                            .ok_or_else(|| value_error("expected a bool or list of bools"))
                    })
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
        other => Err(value_error(format!(
            "closed must be both, left, right or none, not '{other}'"
        ))),
    }
}

pub fn time_unit(s: Option<String>) -> PhpResult<TimeUnit> {
    match s.as_deref().unwrap_or("us") {
        "ns" => Ok(TimeUnit::Nanoseconds),
        "us" => Ok(TimeUnit::Microseconds),
        "ms" => Ok(TimeUnit::Milliseconds),
        other => Err(value_error(format!(
            "time unit must be ns, us or ms, not '{other}'"
        ))),
    }
}

/// A py duration string such as "1d", "2h30m", or "-1w".
pub fn duration(s: &str) -> PhpResult<Duration> {
    use crate::error::OrPhp;
    Duration::try_parse(s).php()
}

/// py closed= for time windows: left, right, both, or none.
pub fn closed_window(s: Option<String>, default: ClosedWindow) -> PhpResult<ClosedWindow> {
    match s.as_deref() {
        None => Ok(default),
        Some("left") => Ok(ClosedWindow::Left),
        Some("right") => Ok(ClosedWindow::Right),
        Some("both") => Ok(ClosedWindow::Both),
        Some("none") => Ok(ClosedWindow::None),
        Some(other) => Err(value_error(format!(
            "closed must be left, right, both or none, not '{other}'"
        ))),
    }
}

/// A search value: a PHP list or Series searches each element, anything else one value.
pub fn collection_or_operand(v: &Zval) -> PhpResult<Expr> {
    if v.array().is_some() || v.extract::<&PhpSeries>().is_some() {
        return literal(v);
    }
    operand(v)
}

/// A range bound: strings name columns, other values are literals.
pub fn operand_or_column(v: &Zval) -> PhpResult<Expr> {
    into_expr(v)
}

pub fn time_zone(s: Option<String>) -> PhpResult<Option<TimeZone>> {
    use crate::error::OrPhp;
    s.map(|z| TimeZone::opt_try_new(Some(z)).php())
        .transpose()
        .map(Option::flatten)
}
