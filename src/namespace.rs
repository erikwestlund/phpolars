#![allow(non_snake_case)]
//! The str, dt, list, struct and name namespaces. Each one wraps an Expr; when
//! reached from a Series it evaluates straight away and returns a Series.
use ext_php_rs::{convert::IntoZval, prelude::*, types::Zval};
use polars::prelude::*;

use crate::{
    dtype,
    error::{OrPhp, value_error},
    expr::wrap,
    input::{into_expr, operand, time_unit},
    series::{PhpSeries, eval},
};

macro_rules! namespace {
    ($rust:ident, $php:literal) => {
        #[php_class]
        #[php(name = $php)]
        pub struct $rust {
            e: Expr,
            s: Option<Series>,
        }

        impl $rust {
            pub fn new(e: Expr, s: Option<Series>) -> Self {
                Self { e, s }
            }

            fn base(&self) -> Expr {
                self.e.clone()
            }

            fn finish(&self, e: Expr) -> PhpResult<Zval> {
                match &self.s {
                    Some(s) => Ok(PhpSeries { inner: eval(s, e)? }.into_zval(false)?),
                    None => Ok(wrap(e).into_zval(false)?),
                }
            }
        }
    };
}

namespace!(PhpStrNameSpace, "Phpolars\\StrNameSpace");
namespace!(PhpDtNameSpace, "Phpolars\\DtNameSpace");
namespace!(PhpListNameSpace, "Phpolars\\ListNameSpace");
namespace!(PhpStructNameSpace, "Phpolars\\StructNameSpace");

fn strptime(format: Option<String>, strict: Option<bool>, exact: Option<bool>) -> StrptimeOptions {
    StrptimeOptions {
        format: format.map(Into::into),
        strict: strict.unwrap_or(true),
        exact: exact.unwrap_or(true),
        cache: true,
    }
}

fn opt_len(length: Option<i64>) -> Expr {
    match length {
        Some(l) => lit(l as u64),
        None => lit(Null {}),
    }
}

fn fill_char(c: Option<String>) -> PhpResult<char> {
    let c = c.unwrap_or_else(|| " ".into());
    let mut chars = c.chars();
    match (chars.next(), chars.next()) {
        (Some(ch), None) => Ok(ch),
        _ => Err(value_error("fillChar must be a single character")),
    }
}

#[php_impl]
impl PhpStrNameSpace {
    pub fn lenChars(&self) -> PhpResult<Zval> {
        self.finish(self.base().str().len_chars())
    }

    pub fn lenBytes(&self) -> PhpResult<Zval> {
        self.finish(self.base().str().len_bytes())
    }

    pub fn toUppercase(&self) -> PhpResult<Zval> {
        self.finish(self.base().str().to_uppercase())
    }

    pub fn toLowercase(&self) -> PhpResult<Zval> {
        self.finish(self.base().str().to_lowercase())
    }

    /// Strip whitespace, or the given characters.
    pub fn stripChars(&self, characters: Option<String>) -> PhpResult<Zval> {
        let m = characters.map(lit).unwrap_or_else(|| lit(Null {}));
        self.finish(self.base().str().strip_chars(m))
    }

    pub fn stripCharsStart(&self, characters: Option<String>) -> PhpResult<Zval> {
        let m = characters.map(lit).unwrap_or_else(|| lit(Null {}));
        self.finish(self.base().str().strip_chars_start(m))
    }

    pub fn stripCharsEnd(&self, characters: Option<String>) -> PhpResult<Zval> {
        let m = characters.map(lit).unwrap_or_else(|| lit(Null {}));
        self.finish(self.base().str().strip_chars_end(m))
    }

    pub fn stripPrefix(&self, prefix: &Zval) -> PhpResult<Zval> {
        self.finish(self.base().str().strip_prefix(operand(prefix)?))
    }

    pub fn stripSuffix(&self, suffix: &Zval) -> PhpResult<Zval> {
        self.finish(self.base().str().strip_suffix(operand(suffix)?))
    }

    pub fn startsWith(&self, prefix: &Zval) -> PhpResult<Zval> {
        self.finish(self.base().str().starts_with(operand(prefix)?))
    }

    pub fn endsWith(&self, suffix: &Zval) -> PhpResult<Zval> {
        self.finish(self.base().str().ends_with(operand(suffix)?))
    }

    /// Regex match unless literal: true.
    pub fn contains(
        &self,
        pattern: &Zval,
        literal: Option<bool>,
        strict: Option<bool>,
    ) -> PhpResult<Zval> {
        let p = operand(pattern)?;
        self.finish(if literal.unwrap_or(false) {
            self.base().str().contains_literal(p)
        } else {
            self.base().str().contains(p, strict.unwrap_or(true))
        })
    }

    pub fn find(
        &self,
        pattern: &Zval,
        literal: Option<bool>,
        strict: Option<bool>,
    ) -> PhpResult<Zval> {
        let p = operand(pattern)?;
        self.finish(if literal.unwrap_or(false) {
            self.base().str().find_literal(p)
        } else {
            self.base().str().find(p, strict.unwrap_or(true))
        })
    }

    pub fn countMatches(&self, pattern: &Zval, literal: Option<bool>) -> PhpResult<Zval> {
        self.finish(
            self.base()
                .str()
                .count_matches(operand(pattern)?, literal.unwrap_or(false)),
        )
    }

    /// Replace the first n matches (default 1).
    pub fn replace(
        &self,
        pattern: &Zval,
        value: &Zval,
        literal: Option<bool>,
        n: Option<i64>,
    ) -> PhpResult<Zval> {
        self.finish(self.base().str().replace_n(
            operand(pattern)?,
            operand(value)?,
            literal.unwrap_or(false),
            n.unwrap_or(1),
        ))
    }

    pub fn replaceAll(
        &self,
        pattern: &Zval,
        value: &Zval,
        literal: Option<bool>,
    ) -> PhpResult<Zval> {
        self.finish(self.base().str().replace_all(
            operand(pattern)?,
            operand(value)?,
            literal.unwrap_or(false),
        ))
    }

    /// Offset and length may be ints or expressions; a null length runs to the end.
    pub fn slice(&self, offset: &Zval, length: Option<&Zval>) -> PhpResult<Zval> {
        let length = match crate::input::given(length) {
            Some(l) => operand(l)?,
            None => lit(Null {}),
        };
        self.finish(self.base().str().slice(operand(offset)?, length))
    }

    pub fn head(&self, n: i64) -> PhpResult<Zval> {
        self.finish(self.base().str().head(lit(n)))
    }

    pub fn tail(&self, n: i64) -> PhpResult<Zval> {
        self.finish(self.base().str().tail(lit(n)))
    }

    /// Split on a literal separator, or on a regex with literal: false.
    pub fn split(
        &self,
        by: &Zval,
        inclusive: Option<bool>,
        literal: Option<bool>,
    ) -> PhpResult<Zval> {
        let by = operand(by)?;
        let inclusive = inclusive.unwrap_or(false);
        self.finish(match (literal.unwrap_or(true), inclusive) {
            (true, false) => self.base().str().split(by),
            (true, true) => self.base().str().split_inclusive(by),
            (false, false) => self.base().str().split_regex(by, true),
            (false, true) => self.base().str().split_regex_inclusive(by, true),
        })
    }

    pub fn splitn(&self, by: &Zval, n: i64) -> PhpResult<Zval> {
        if n < 1 {
            return Err(value_error(format!("n must be at least 1, got {n}")));
        }
        self.finish(self.base().str().splitn(operand(by)?, n as usize))
    }

    /// Pad to a length, which may be an int or an expression.
    pub fn padStart(&self, length: &Zval, fillChar: Option<String>) -> PhpResult<Zval> {
        let length = operand(length)?.cast(DataType::UInt64);
        self.finish(self.base().str().pad_start(length, fill_char(fillChar)?))
    }

    pub fn padEnd(&self, length: &Zval, fillChar: Option<String>) -> PhpResult<Zval> {
        let length = operand(length)?.cast(DataType::UInt64);
        self.finish(self.base().str().pad_end(length, fill_char(fillChar)?))
    }

    pub fn zfill(&self, length: i64) -> PhpResult<Zval> {
        self.finish(
            self.base()
                .str()
                .zfill(lit(crate::input::count(length, "length")? as u64)),
        )
    }

    pub fn reverse(&self) -> PhpResult<Zval> {
        self.finish(self.base().str().reverse())
    }

    pub fn extract(&self, pattern: &Zval, groupIndex: Option<i64>) -> PhpResult<Zval> {
        self.finish(self.base().str().extract(
            operand(pattern)?,
            crate::input::count(groupIndex.unwrap_or(1), "groupIndex")?,
        ))
    }

    pub fn extractAll(&self, pattern: &Zval) -> PhpResult<Zval> {
        self.finish(self.base().str().extract_all(operand(pattern)?))
    }

    /// Concatenate all values into one string.
    pub fn join(&self, delimiter: Option<String>, ignoreNulls: Option<bool>) -> PhpResult<Zval> {
        self.finish(
            self.base()
                .str()
                .join(&delimiter.unwrap_or_default(), ignoreNulls.unwrap_or(true)),
        )
    }

    pub fn toInteger(
        &self,
        base: Option<i64>,
        dtype: Option<String>,
        strict: Option<bool>,
    ) -> PhpResult<Zval> {
        let dt = dtype.map(|d| dtype::parse(&d)).transpose()?;
        self.finish(self.base().str().to_integer(
            lit(base.unwrap_or(10) as u32),
            dt,
            strict.unwrap_or(true),
        ))
    }

    pub fn toDate(
        &self,
        format: Option<String>,
        strict: Option<bool>,
        exact: Option<bool>,
    ) -> PhpResult<Zval> {
        self.finish(self.base().str().to_date(strptime(format, strict, exact)))
    }

    pub fn toDatetime(
        &self,
        format: Option<String>,
        timeUnit: Option<String>,
        timeZone: Option<String>,
        strict: Option<bool>,
        exact: Option<bool>,
    ) -> PhpResult<Zval> {
        let tu = timeUnit.map(|t| time_unit(Some(t))).transpose()?;
        let tz = TimeZone::opt_try_new(timeZone).php()?;
        self.finish(self.base().str().to_datetime(
            tu,
            tz,
            strptime(format, strict, exact),
            lit("raise"),
        ))
    }

    pub fn toTime(&self, format: Option<String>, strict: Option<bool>) -> PhpResult<Zval> {
        self.finish(
            self.base()
                .str()
                .to_time(strptime(format, strict, Some(true))),
        )
    }

    pub fn strptime(
        &self,
        dtype: String,
        format: Option<String>,
        strict: Option<bool>,
        exact: Option<bool>,
    ) -> PhpResult<Zval> {
        let dt = dtype::parse(&dtype)?;
        self.finish(
            self.base()
                .str()
                .strptime(dt, strptime(format, strict, exact), lit("raise")),
        )
    }
}

#[php_impl]
impl PhpDtNameSpace {
    pub fn year(&self) -> PhpResult<Zval> {
        self.finish(self.base().dt().year())
    }

    pub fn isoYear(&self) -> PhpResult<Zval> {
        self.finish(self.base().dt().iso_year())
    }

    pub fn quarter(&self) -> PhpResult<Zval> {
        self.finish(self.base().dt().quarter())
    }

    pub fn month(&self) -> PhpResult<Zval> {
        self.finish(self.base().dt().month())
    }

    pub fn week(&self) -> PhpResult<Zval> {
        self.finish(self.base().dt().week())
    }

    /// ISO weekday, Monday = 1.
    pub fn weekday(&self) -> PhpResult<Zval> {
        self.finish(self.base().dt().weekday())
    }

    pub fn day(&self) -> PhpResult<Zval> {
        self.finish(self.base().dt().day())
    }

    pub fn ordinalDay(&self) -> PhpResult<Zval> {
        self.finish(self.base().dt().ordinal_day())
    }

    pub fn hour(&self) -> PhpResult<Zval> {
        self.finish(self.base().dt().hour())
    }

    pub fn minute(&self) -> PhpResult<Zval> {
        self.finish(self.base().dt().minute())
    }

    pub fn second(&self) -> PhpResult<Zval> {
        self.finish(self.base().dt().second())
    }

    pub fn millisecond(&self) -> PhpResult<Zval> {
        self.finish(self.base().dt().millisecond())
    }

    pub fn microsecond(&self) -> PhpResult<Zval> {
        self.finish(self.base().dt().microsecond())
    }

    pub fn nanosecond(&self) -> PhpResult<Zval> {
        self.finish(self.base().dt().nanosecond())
    }

    pub fn isLeapYear(&self) -> PhpResult<Zval> {
        self.finish(self.base().dt().is_leap_year())
    }

    pub fn date(&self) -> PhpResult<Zval> {
        self.finish(self.base().dt().date())
    }

    pub fn time(&self) -> PhpResult<Zval> {
        self.finish(self.base().dt().time())
    }

    pub fn timestamp(&self, timeUnit: Option<String>) -> PhpResult<Zval> {
        self.finish(self.base().dt().timestamp(time_unit(timeUnit)?))
    }

    /// Time since the epoch: 'ns', 'us', 'ms', 's' or 'd'.
    pub fn epoch(&self, timeUnit: Option<String>) -> PhpResult<Zval> {
        let e = match timeUnit.as_deref().unwrap_or("us") {
            "s" => self
                .base()
                .dt()
                .timestamp(TimeUnit::Milliseconds)
                .floor_div(lit(1000i64)),
            "d" => self.base().cast(DataType::Date).cast(DataType::Int32),
            other => self.base().dt().timestamp(time_unit(Some(other.into()))?),
        };
        self.finish(e)
    }

    pub fn toString(&self, format: Option<String>) -> PhpResult<Zval> {
        self.finish(
            self.base()
                .dt()
                .to_string(&format.unwrap_or_else(|| "iso".into())),
        )
    }

    pub fn strftime(&self, format: String) -> PhpResult<Zval> {
        self.finish(self.base().dt().strftime(&format))
    }

    /// Truncate to a calendar interval such as '1d', '1mo', '15m'.
    pub fn truncate(&self, every: String) -> PhpResult<Zval> {
        self.finish(self.base().dt().truncate(lit(every)))
    }

    pub fn round(&self, every: String) -> PhpResult<Zval> {
        self.finish(self.base().dt().round(lit(every)))
    }

    pub fn offsetBy(&self, by: String) -> PhpResult<Zval> {
        self.finish(self.base().dt().offset_by(lit(by)))
    }

    pub fn monthStart(&self) -> PhpResult<Zval> {
        self.finish(self.base().dt().month_start())
    }

    pub fn monthEnd(&self) -> PhpResult<Zval> {
        self.finish(self.base().dt().month_end())
    }

    pub fn castTimeUnit(&self, timeUnit: String) -> PhpResult<Zval> {
        self.finish(self.base().dt().cast_time_unit(time_unit(Some(timeUnit))?))
    }

    pub fn convertTimeZone(&self, timeZone: String) -> PhpResult<Zval> {
        let tz = TimeZone::opt_try_new(Some(timeZone)).php()?.unwrap();
        self.finish(self.base().dt().convert_time_zone(tz))
    }

    /// ambiguous: raise, earliest, latest or null; nonExistent: raise or null.
    pub fn replaceTimeZone(
        &self,
        timeZone: Option<String>,
        ambiguous: Option<String>,
        nonExistent: Option<String>,
    ) -> PhpResult<Zval> {
        let tz = TimeZone::opt_try_new(timeZone).php()?;
        let non_existent = match nonExistent.as_deref().unwrap_or("raise") {
            "raise" => NonExistent::Raise,
            "null" => NonExistent::Null,
            other => {
                return Err(value_error(format!(
                    "nonExistent must be raise or null, not '{other}'"
                )));
            }
        };
        let ambiguous = ambiguous.unwrap_or_else(|| "raise".into());
        self.finish(
            self.base()
                .dt()
                .replace_time_zone(tz, lit(ambiguous), non_existent),
        )
    }

    pub fn totalDays(&self, fractional: Option<bool>) -> PhpResult<Zval> {
        self.finish(self.base().dt().total_days(fractional.unwrap_or(false)))
    }

    pub fn totalHours(&self, fractional: Option<bool>) -> PhpResult<Zval> {
        self.finish(self.base().dt().total_hours(fractional.unwrap_or(false)))
    }

    pub fn totalMinutes(&self, fractional: Option<bool>) -> PhpResult<Zval> {
        self.finish(self.base().dt().total_minutes(fractional.unwrap_or(false)))
    }

    pub fn totalSeconds(&self, fractional: Option<bool>) -> PhpResult<Zval> {
        self.finish(self.base().dt().total_seconds(fractional.unwrap_or(false)))
    }

    pub fn totalMilliseconds(&self, fractional: Option<bool>) -> PhpResult<Zval> {
        self.finish(
            self.base()
                .dt()
                .total_milliseconds(fractional.unwrap_or(false)),
        )
    }

    pub fn totalMicroseconds(&self, fractional: Option<bool>) -> PhpResult<Zval> {
        self.finish(
            self.base()
                .dt()
                .total_microseconds(fractional.unwrap_or(false)),
        )
    }
}

#[php_impl]
impl PhpListNameSpace {
    pub fn len(&self) -> PhpResult<Zval> {
        self.finish(self.base().list().len())
    }

    pub fn sum(&self) -> PhpResult<Zval> {
        self.finish(self.base().list().sum())
    }

    pub fn mean(&self) -> PhpResult<Zval> {
        self.finish(self.base().list().mean())
    }

    pub fn median(&self) -> PhpResult<Zval> {
        self.finish(self.base().list().median())
    }

    pub fn min(&self) -> PhpResult<Zval> {
        self.finish(self.base().list().min())
    }

    pub fn max(&self) -> PhpResult<Zval> {
        self.finish(self.base().list().max())
    }

    pub fn std(&self, ddof: Option<i64>) -> PhpResult<Zval> {
        self.finish(self.base().list().std(crate::input::ddof(ddof)?))
    }

    pub fn var(&self, ddof: Option<i64>) -> PhpResult<Zval> {
        self.finish(self.base().list().var(crate::input::ddof(ddof)?))
    }

    pub fn first(&self) -> PhpResult<Zval> {
        self.finish(self.base().list().first())
    }

    pub fn last(&self) -> PhpResult<Zval> {
        self.finish(self.base().list().last())
    }

    pub fn get(&self, index: &Zval, nullOnOob: Option<bool>) -> PhpResult<Zval> {
        self.finish(
            self.base()
                .list()
                .get(operand(index)?, nullOnOob.unwrap_or(false)),
        )
    }

    pub fn gather(&self, indices: &Zval, nullOnOob: Option<bool>) -> PhpResult<Zval> {
        self.finish(
            self.base()
                .list()
                .gather(operand(indices)?, nullOnOob.unwrap_or(false)),
        )
    }

    pub fn contains(&self, item: &Zval, nullsEqual: Option<bool>) -> PhpResult<Zval> {
        self.finish(
            self.base()
                .list()
                .contains(operand(item)?, nullsEqual.unwrap_or(true)),
        )
    }

    pub fn countMatches(&self, element: &Zval) -> PhpResult<Zval> {
        self.finish(self.base().list().count_matches(operand(element)?))
    }

    pub fn join(&self, separator: &Zval, ignoreNulls: Option<bool>) -> PhpResult<Zval> {
        self.finish(
            self.base()
                .list()
                .join(operand(separator)?, ignoreNulls.unwrap_or(true)),
        )
    }

    pub fn argMin(&self) -> PhpResult<Zval> {
        self.finish(self.base().list().arg_min())
    }

    pub fn argMax(&self) -> PhpResult<Zval> {
        self.finish(self.base().list().arg_max())
    }

    pub fn sort(&self, descending: Option<bool>, nullsLast: Option<bool>) -> PhpResult<Zval> {
        let options = SortOptions::default()
            .with_order_descending(descending.unwrap_or(false))
            .with_nulls_last(nullsLast.unwrap_or(false));
        self.finish(self.base().list().sort(options))
    }

    pub fn reverse(&self) -> PhpResult<Zval> {
        self.finish(self.base().list().eval(element().reverse()))
    }

    pub fn unique(&self, maintainOrder: Option<bool>) -> PhpResult<Zval> {
        let u = if maintainOrder.unwrap_or(false) {
            element().unique_stable()
        } else {
            element().unique()
        };
        self.finish(self.base().list().eval(u))
    }

    pub fn nUnique(&self) -> PhpResult<Zval> {
        self.finish(self.base().list().eval(element().n_unique()).list().first())
    }

    pub fn dropNulls(&self) -> PhpResult<Zval> {
        self.finish(self.base().list().drop_nulls())
    }

    pub fn slice(&self, offset: i64, length: Option<i64>) -> PhpResult<Zval> {
        self.finish(self.base().list().slice(lit(offset), opt_len(length)))
    }

    pub fn head(&self, n: Option<i64>) -> PhpResult<Zval> {
        self.finish(self.base().list().head(lit(n.unwrap_or(5))))
    }

    pub fn tail(&self, n: Option<i64>) -> PhpResult<Zval> {
        self.finish(self.base().list().tail(lit(n.unwrap_or(5))))
    }

    pub fn shift(&self, n: Option<i64>) -> PhpResult<Zval> {
        self.finish(self.base().list().shift(lit(n.unwrap_or(1))))
    }

    pub fn diff(&self, n: Option<i64>) -> PhpResult<Zval> {
        self.finish(self.base().list().diff(
            n.unwrap_or(1),
            polars_core::series::ops::NullBehavior::Ignore,
        ))
    }

    pub fn concat(&self, other: &Zval) -> PhpResult<Zval> {
        let e = concat_list([self.base(), into_expr(other)?]).php()?;
        self.finish(e)
    }

    /// Run an expression on each list; refer to the elements with element().
    pub fn eval(&self, expr: &Zval) -> PhpResult<Zval> {
        self.finish(self.base().list().eval(into_expr(expr)?))
    }

    pub fn setUnion(&self, other: &Zval) -> PhpResult<Zval> {
        self.finish(self.base().list().union(into_expr(other)?))
    }

    pub fn setIntersection(&self, other: &Zval) -> PhpResult<Zval> {
        self.finish(self.base().list().set_intersection(into_expr(other)?))
    }

    pub fn setDifference(&self, other: &Zval) -> PhpResult<Zval> {
        self.finish(self.base().list().set_difference(into_expr(other)?))
    }

    pub fn setSymmetricDifference(&self, other: &Zval) -> PhpResult<Zval> {
        self.finish(
            self.base()
                .list()
                .set_symmetric_difference(into_expr(other)?),
        )
    }

    pub fn explode(&self) -> PhpResult<Zval> {
        self.finish(self.base().explode(ExplodeOptions {
            empty_as_null: true,
            keep_nulls: true,
        }))
    }

    pub fn toStruct(&self, fields: Option<&Zval>) -> PhpResult<Zval> {
        let names: Vec<PlSmallStr> = match crate::input::given(fields) {
            Some(f) => crate::input::names(&[f])?
                .into_iter()
                .map(Into::into)
                .collect(),
            None => return Err(value_error("toStruct() needs the field names")),
        };
        self.finish(self.base().list().to_struct(names.into()))
    }
}

#[php_impl]
impl PhpStructNameSpace {
    pub fn field(&self, name: String) -> PhpResult<Zval> {
        self.finish(self.base().struct_().field_by_name(&name))
    }

    pub fn renameFields(&self, names: &Zval) -> PhpResult<Zval> {
        let names = crate::input::names(&[names])?;
        self.finish(self.base().struct_().rename_fields(names))
    }

    pub fn jsonEncode(&self) -> PhpResult<Zval> {
        #[cfg(feature = "json")]
        {
            self.finish(self.base().struct_().json_encode())
        }
        #[cfg(not(feature = "json"))]
        Err(PhpException::from_class::<crate::error::ComputeError>(
            "php_polars was built without the 'json' feature".into(),
        ))
    }
}

/// Output-name changes; only meaningful on expressions.
#[php_class]
#[php(name = "Phpolars\\NameNameSpace")]
pub struct PhpNameNameSpace {
    e: Expr,
}

impl PhpNameNameSpace {
    pub fn new(e: Expr) -> Self {
        Self { e }
    }
}

#[php_impl]
impl PhpNameNameSpace {
    pub fn keep(&self) -> crate::expr::PhpExpr {
        wrap(self.e.clone().name().keep())
    }

    pub fn prefix(&self, prefix: String) -> crate::expr::PhpExpr {
        wrap(self.e.clone().name().prefix(&prefix))
    }

    pub fn suffix(&self, suffix: String) -> crate::expr::PhpExpr {
        wrap(self.e.clone().name().suffix(&suffix))
    }

    pub fn toLowercase(&self) -> crate::expr::PhpExpr {
        wrap(self.e.clone().name().to_lowercase())
    }

    pub fn toUppercase(&self) -> crate::expr::PhpExpr {
        wrap(self.e.clone().name().to_uppercase())
    }

    pub fn replace(
        &self,
        pattern: String,
        value: String,
        literal: Option<bool>,
    ) -> crate::expr::PhpExpr {
        wrap(
            self.e
                .clone()
                .name()
                .replace(&pattern, &value, literal.unwrap_or(false)),
        )
    }
}
