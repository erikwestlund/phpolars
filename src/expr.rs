use ext_php_rs::{prelude::*, types::Zval};
use polars::prelude::{self as pl, Expr, LiteralValue, Scalar};

/// A Polars expression built from column names and literals. Expressions
/// never hold PHP closures, so they can run on Polars worker threads.
#[php_class]
#[php(name = "Phpolars\\Expr")]
#[derive(Clone)]
pub struct PhpExpr {
    pub(crate) inner: Expr,
}

fn wrap(inner: Expr) -> PhpExpr {
    PhpExpr { inner }
}

pub(crate) fn literal(value: &Zval) -> PhpResult<Expr> {
    if value.is_null() {
        Ok(pl::lit(LiteralValue::Scalar(Scalar::null(pl::DataType::Null))))
    } else if let Some(v) = value.bool() {
        Ok(pl::lit(v))
    } else if let Some(v) = value.long() {
        Ok(pl::lit(v))
    } else if let Some(v) = value.double() {
        Ok(pl::lit(v))
    } else if let Some(v) = value.str() {
        Ok(pl::lit(v.to_owned()))
    } else {
        Err("literals must be null, bool, int, float or string".into())
    }
}

/// An Expr passes through; anything else becomes a literal.
pub(crate) fn operand(value: &Zval) -> PhpResult<Expr> {
    match value.extract::<&PhpExpr>() {
        Some(e) => Ok(e.inner.clone()),
        None => literal(value),
    }
}

#[php_impl]
impl PhpExpr {
    pub fn col(name: String) -> Self {
        wrap(pl::col(name))
    }

    pub fn lit(value: &Zval) -> PhpResult<Self> {
        literal(value).map(wrap)
    }

    pub fn eq(&self, other: &Zval) -> PhpResult<Self> {
        Ok(wrap(self.inner.clone().eq(operand(other)?)))
    }

    pub fn neq(&self, other: &Zval) -> PhpResult<Self> {
        Ok(wrap(self.inner.clone().neq(operand(other)?)))
    }

    pub fn gt(&self, other: &Zval) -> PhpResult<Self> {
        Ok(wrap(self.inner.clone().gt(operand(other)?)))
    }

    pub fn gte(&self, other: &Zval) -> PhpResult<Self> {
        Ok(wrap(self.inner.clone().gt_eq(operand(other)?)))
    }

    pub fn lt(&self, other: &Zval) -> PhpResult<Self> {
        Ok(wrap(self.inner.clone().lt(operand(other)?)))
    }

    pub fn lte(&self, other: &Zval) -> PhpResult<Self> {
        Ok(wrap(self.inner.clone().lt_eq(operand(other)?)))
    }

    pub fn and(&self, other: &Zval) -> PhpResult<Self> {
        Ok(wrap(self.inner.clone().and(operand(other)?)))
    }

    pub fn or(&self, other: &Zval) -> PhpResult<Self> {
        Ok(wrap(self.inner.clone().or(operand(other)?)))
    }

    pub fn not(&self) -> Self {
        wrap(self.inner.clone().not())
    }

    pub fn is_null(&self) -> Self {
        wrap(self.inner.clone().is_null())
    }

    pub fn add(&self, other: &Zval) -> PhpResult<Self> {
        Ok(wrap(self.inner.clone() + operand(other)?))
    }

    pub fn sub(&self, other: &Zval) -> PhpResult<Self> {
        Ok(wrap(self.inner.clone() - operand(other)?))
    }

    pub fn mul(&self, other: &Zval) -> PhpResult<Self> {
        Ok(wrap(self.inner.clone() * operand(other)?))
    }

    pub fn div(&self, other: &Zval) -> PhpResult<Self> {
        Ok(wrap(self.inner.clone() / operand(other)?))
    }

    pub fn mean(&self) -> Self {
        wrap(self.inner.clone().mean())
    }

    pub fn sum(&self) -> Self {
        wrap(self.inner.clone().sum())
    }

    /// Non-null values, as in SQL COUNT(col).
    pub fn count(&self) -> Self {
        wrap(self.inner.clone().count())
    }

    pub fn min(&self) -> Self {
        wrap(self.inner.clone().min())
    }

    pub fn max(&self) -> Self {
        wrap(self.inner.clone().max())
    }

    pub fn alias(&self, name: String) -> Self {
        wrap(self.inner.clone().alias(name))
    }
}
