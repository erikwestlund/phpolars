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
namespace!(PhpCatNameSpace, "Phpolars\\CatNameSpace");
namespace!(PhpArrNameSpace, "Phpolars\\ArrayNameSpace");
namespace!(PhpBinNameSpace, "Phpolars\\BinaryNameSpace");

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

/// Patterns for the aho-corasick str methods: a list of strings, one
/// string (a literal, as in py), or an Expr.
fn patterns_expr(v: &Zval) -> PhpResult<Expr> {
    if v.array().is_some() {
        return Ok(crate::convert::literal(v)?.implode(true));
    }
    operand(v)
}

fn week_mask(mask: Option<Vec<bool>>) -> PhpResult<[bool; 7]> {
    match mask {
        None => Ok([true, true, true, true, true, false, false]),
        Some(m) => m
            .try_into()
            .map_err(|_| value_error("weekMask needs seven booleans, Monday to Sunday")),
    }
}

fn holidays_expr(v: Option<&Zval>) -> PhpResult<Expr> {
    match crate::input::given(v) {
        None => Ok(lit(Series::new_empty("".into(), &DataType::Date)).implode(true)),
        Some(v) if v.array().is_some() => Ok(crate::convert::literal(v)?
            .cast(DataType::Date)
            .implode(true)),
        Some(v) => into_expr(v),
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
    /// True where any of the literal patterns occurs.
    pub fn containsAny(
        &self,
        patterns: &Zval,
        asciiCaseInsensitive: Option<bool>,
    ) -> PhpResult<Zval> {
        self.finish(self.base().str().contains_any(
            patterns_expr(patterns)?,
            asciiCaseInsensitive.unwrap_or(false),
        ))
    }

    /// replaceMany(['a', 'b'], 'x') or replaceMany(['a' => 'x', 'b' => 'y']).
    pub fn replaceMany(
        &self,
        patterns: &Zval,
        replaceWith: Option<&Zval>,
        asciiCaseInsensitive: Option<bool>,
        leftmost: Option<bool>,
    ) -> PhpResult<Zval> {
        let (p, r) = match crate::input::given(replaceWith) {
            // One replacement for every pattern goes in as a one-item list.
            Some(r) if r.str().is_some() => (
                patterns_expr(patterns)?,
                lit(Series::new("".into(), [r.str().unwrap()])).implode(true),
            ),
            Some(r) => (patterns_expr(patterns)?, patterns_expr(r)?),
            None => {
                let map = patterns.array().filter(|a| !crate::convert::is_list(a)).ok_or_else(|| {
                    value_error("replaceMany() takes patterns and replaceWith, or a ['pattern' => 'replacement'] array")
                })?;
                let (mut olds, mut news) = (Vec::new(), Vec::new());
                for (k, v) in map.iter() {
                    olds.push(k.to_string());
                    news.push(
                        v.str()
                            .ok_or_else(|| value_error("replacements must be strings"))?
                            .to_owned(),
                    );
                }
                (
                    lit(Series::new("".into(), olds)).implode(true),
                    lit(Series::new("".into(), news)).implode(true),
                )
            }
        };
        self.finish(self.base().str().replace_many(
            p,
            r,
            asciiCaseInsensitive.unwrap_or(false),
            leftmost.unwrap_or(false),
        ))
    }

    pub fn extractMany(
        &self,
        patterns: &Zval,
        asciiCaseInsensitive: Option<bool>,
        overlapping: Option<bool>,
        leftmost: Option<bool>,
    ) -> PhpResult<Zval> {
        self.finish(self.base().str().extract_many(
            patterns_expr(patterns)?,
            asciiCaseInsensitive.unwrap_or(false),
            overlapping.unwrap_or(false),
            leftmost.unwrap_or(false),
        ))
    }

    pub fn findMany(
        &self,
        patterns: &Zval,
        asciiCaseInsensitive: Option<bool>,
        overlapping: Option<bool>,
        leftmost: Option<bool>,
    ) -> PhpResult<Zval> {
        self.finish(self.base().str().find_many(
            patterns_expr(patterns)?,
            asciiCaseInsensitive.unwrap_or(false),
            overlapping.unwrap_or(false),
            leftmost.unwrap_or(false),
        ))
    }

    /// The regex capture groups as a struct, named fields by name and others by position.
    pub fn extractGroups(&self, pattern: String) -> PhpResult<Zval> {
        self.finish(self.base().str().extract_groups(&pattern).php()?)
    }

    /// Split into a struct of n + 1 fields: field_0, field_1, ...
    pub fn splitExact(&self, by: &Zval, n: i64, inclusive: Option<bool>) -> PhpResult<Zval> {
        let n = crate::input::count(n, "n")?;
        let by = operand(by)?;
        self.finish(if inclusive.unwrap_or(false) {
            self.base().str().split_exact_inclusive(by, n)
        } else {
            self.base().str().split_exact(by, n)
        })
    }

    /// 'hex' or 'base64'.
    pub fn encode(&self, encoding: String) -> PhpResult<Zval> {
        self.finish(match encoding.as_str() {
            "hex" => self.base().str().hex_encode(),
            "base64" => self.base().str().base64_encode(),
            e => {
                return Err(value_error(format!(
                    "encoding must be hex or base64, not '{e}'"
                )));
            }
        })
    }

    /// Decode 'hex' or 'base64' text to Binary.
    pub fn decode(&self, encoding: String, strict: Option<bool>) -> PhpResult<Zval> {
        let strict = strict.unwrap_or(true);
        self.finish(match encoding.as_str() {
            "hex" => self.base().str().hex_decode(strict),
            "base64" => self.base().str().base64_decode(strict),
            e => {
                return Err(value_error(format!(
                    "encoding must be hex or base64, not '{e}'"
                )));
            }
        })
    }

    pub fn toDecimal(&self, scale: i64) -> PhpResult<Zval> {
        self.finish(
            self.base()
                .str()
                .to_decimal(crate::input::count(scale, "scale")?),
        )
    }

    /// Unicode normalization: NFC, NFKC, NFD, or NFKD.
    pub fn normalize(&self, form: Option<String>) -> PhpResult<Zval> {
        let form = match form.as_deref().unwrap_or("NFC") {
            "NFC" => UnicodeForm::NFC,
            "NFKC" => UnicodeForm::NFKC,
            "NFD" => UnicodeForm::NFD,
            "NFKD" => UnicodeForm::NFKD,
            f => {
                return Err(value_error(format!(
                    "form must be NFC, NFKC, NFD, or NFKD, not '{f}'"
                )));
            }
        };
        self.finish(self.base().str().normalize(form))
    }

    /// Parse JSON text into the given type, e.g. 'Struct(a: Int64)' or 'List(String)'.
    pub fn jsonDecode(&self, dtype: String) -> PhpResult<Zval> {
        self.finish(self.base().str().json_decode(dtype::parse(&dtype)?))
    }

    /// The first match of a JSONPath such as '$.a.b', as a string.
    pub fn jsonPathMatch(&self, jsonPath: &Zval) -> PhpResult<Zval> {
        self.finish(self.base().str().json_path_match(operand(jsonPath)?))
    }

    pub fn escapeRegex(&self) -> PhpResult<Zval> {
        self.finish(self.base().str().escape_regex())
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
    pub fn millennium(&self) -> PhpResult<Zval> {
        self.finish(self.base().dt().millennium())
    }

    pub fn century(&self) -> PhpResult<Zval> {
        self.finish(self.base().dt().century())
    }

    pub fn daysInMonth(&self) -> PhpResult<Zval> {
        self.finish(self.base().dt().days_in_month())
    }

    /// Reinterpret the stored integers in another unit (castTimeUnit converts).
    pub fn withTimeUnit(&self, timeUnit: String) -> PhpResult<Zval> {
        self.finish(self.base().dt().with_time_unit(time_unit(Some(timeUnit))?))
    }

    pub fn baseUtcOffset(&self) -> PhpResult<Zval> {
        self.finish(self.base().dt().base_utc_offset())
    }

    pub fn dstOffset(&self) -> PhpResult<Zval> {
        self.finish(self.base().dt().dst_offset())
    }

    pub fn totalNanoseconds(&self, fractional: Option<bool>) -> PhpResult<Zval> {
        self.finish(
            self.base()
                .dt()
                .total_nanoseconds(fractional.unwrap_or(false)),
        )
    }

    /// A Date with a Time ('HH:MM:SS' text or an Expr) becomes a Datetime.
    pub fn combine(&self, time: &Zval, timeUnit: Option<String>) -> PhpResult<Zval> {
        let t = match time.str() {
            Some(s) => lit(s).str().to_time(StrptimeOptions::default()),
            None => into_expr(time)?,
        };
        self.finish(self.base().dt().combine(t, time_unit(timeUnit)?))
    }

    /// Replace parts of a date or datetime: replace(year: 2025, day: 1).
    pub fn replace(
        &self,
        year: Option<&Zval>,
        month: Option<&Zval>,
        day: Option<&Zval>,
        hour: Option<&Zval>,
        minute: Option<&Zval>,
        second: Option<&Zval>,
        microsecond: Option<&Zval>,
        ambiguous: Option<String>,
    ) -> PhpResult<Zval> {
        let part = |v: Option<&Zval>| -> PhpResult<Expr> {
            match crate::input::given(v) {
                Some(v) => into_expr(v),
                None => Ok(lit(Null {})),
            }
        };
        self.finish(self.base().dt().replace(
            part(year)?,
            part(month)?,
            part(day)?,
            part(hour)?,
            part(minute)?,
            part(second)?,
            part(microsecond)?,
            lit(ambiguous.unwrap_or_else(|| "raise".into())),
        ))
    }

    /// Add business days; weekMask lists Monday..Sunday, holidays are dates.
    pub fn addBusinessDays(
        &self,
        n: &Zval,
        weekMask: Option<Vec<bool>>,
        holidays: Option<&Zval>,
        roll: Option<String>,
    ) -> PhpResult<Zval> {
        let roll = match roll.as_deref().unwrap_or("raise") {
            "raise" => Roll::Raise,
            "forward" => Roll::Forward,
            "backward" => Roll::Backward,
            r => {
                return Err(value_error(format!(
                    "roll must be raise, forward, or backward, not '{r}'"
                )));
            }
        };
        self.finish(self.base().dt().add_business_days(
            into_expr(n)?,
            week_mask(weekMask)?,
            holidays_expr(holidays)?,
            roll,
        ))
    }

    pub fn isBusinessDay(
        &self,
        weekMask: Option<Vec<bool>>,
        holidays: Option<&Zval>,
    ) -> PhpResult<Zval> {
        self.finish(
            self.base()
                .dt()
                .is_business_day(week_mask(weekMask)?, holidays_expr(holidays)?),
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

    /// True where any element is true; nulls are skipped unless ignoreNulls: false.
    pub fn any(&self, ignoreNulls: Option<bool>) -> PhpResult<Zval> {
        self.finish(
            self.base()
                .list()
                .eval(element().any(ignoreNulls.unwrap_or(true)))
                .list()
                .first(),
        )
    }

    pub fn all(&self, ignoreNulls: Option<bool>) -> PhpResult<Zval> {
        self.finish(
            self.base()
                .list()
                .eval(element().all(ignoreNulls.unwrap_or(true)))
                .list()
                .first(),
        )
    }

    /// Keep the elements where predicate (written with element()) is true.
    pub fn filter(&self, predicate: &Zval) -> PhpResult<Zval> {
        self.finish(
            self.base()
                .list()
                .eval(element().filter(into_expr(predicate)?)),
        )
    }

    pub fn sample(
        &self,
        n: Option<&Zval>,
        fraction: Option<&Zval>,
        withReplacement: Option<bool>,
        shuffle: Option<bool>,
        seed: Option<i64>,
    ) -> PhpResult<Zval> {
        let wr = withReplacement.unwrap_or(false);
        let seed = seed.map(|s| s as u64);
        self.finish(
            match (crate::input::given(n), crate::input::given(fraction)) {
                (Some(_), Some(_)) => {
                    return Err(value_error("sample() takes n or fraction, not both"));
                }
                (_, Some(f)) => self
                    .base()
                    .list()
                    .sample_fraction(operand(f)?, wr, shuffle, seed),
                (Some(n), None) => self.base().list().sample_n(operand(n)?, wr, shuffle, seed),
                (None, None) => self.base().list().sample_n(lit(1), wr, shuffle, seed),
            },
        )
    }

    pub fn gatherEvery(&self, n: &Zval, offset: Option<&Zval>) -> PhpResult<Zval> {
        let offset = match crate::input::given(offset) {
            Some(o) => operand(o)?,
            None => lit(0),
        };
        self.finish(self.base().list().gather_every(operand(n)?, offset))
    }

    /// A fixed-width Array column.
    pub fn toArray(&self, width: i64) -> PhpResult<Zval> {
        self.finish(
            self.base()
                .list()
                .to_array(crate::input::count(width, "width")?),
        )
    }

    /// Run an aggregation over each list: agg(element()->sum()).
    pub fn agg(&self, expr: &Zval) -> PhpResult<Zval> {
        self.finish(self.base().list().agg(into_expr(expr)?))
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

#[php_impl]
impl PhpCatNameSpace {
    /// The categories as a String column.
    pub fn getCategories(&self) -> PhpResult<Zval> {
        self.finish(self.base().cat().get_categories())
    }

    pub fn lenBytes(&self) -> PhpResult<Zval> {
        self.finish(self.base().cat().len_bytes())
    }

    pub fn lenChars(&self) -> PhpResult<Zval> {
        self.finish(self.base().cat().len_chars())
    }

    pub fn startsWith(&self, prefix: String) -> PhpResult<Zval> {
        self.finish(self.base().cat().starts_with(prefix))
    }

    pub fn endsWith(&self, suffix: String) -> PhpResult<Zval> {
        self.finish(self.base().cat().ends_with(suffix))
    }

    pub fn slice(&self, offset: i64, length: Option<i64>) -> PhpResult<Zval> {
        let length = length
            .map(|l| crate::input::count(l, "length"))
            .transpose()?;
        self.finish(self.base().cat().slice(offset, length))
    }
}

/// The arr namespace, for fixed-width Array columns.
#[php_impl]
impl PhpArrNameSpace {
    pub fn len(&self) -> PhpResult<Zval> {
        self.finish(self.base().arr().len())
    }

    pub fn max(&self) -> PhpResult<Zval> {
        self.finish(self.base().arr().max())
    }

    pub fn min(&self) -> PhpResult<Zval> {
        self.finish(self.base().arr().min())
    }

    pub fn sum(&self) -> PhpResult<Zval> {
        self.finish(self.base().arr().sum())
    }

    pub fn mean(&self) -> PhpResult<Zval> {
        self.finish(self.base().arr().mean())
    }

    pub fn median(&self) -> PhpResult<Zval> {
        self.finish(self.base().arr().median())
    }

    pub fn std(&self, ddof: Option<i64>) -> PhpResult<Zval> {
        self.finish(self.base().arr().std(crate::input::ddof(ddof)?))
    }

    pub fn var(&self, ddof: Option<i64>) -> PhpResult<Zval> {
        self.finish(self.base().arr().var(crate::input::ddof(ddof)?))
    }

    pub fn unique(&self, maintainOrder: Option<bool>) -> PhpResult<Zval> {
        let u = if maintainOrder.unwrap_or(false) {
            element().unique_stable()
        } else {
            element().unique()
        };
        self.finish(self.base().arr().eval(u, true))
    }

    pub fn nUnique(&self) -> PhpResult<Zval> {
        self.finish(self.base().arr().agg(element().n_unique()))
    }

    pub fn any(&self, ignoreNulls: Option<bool>) -> PhpResult<Zval> {
        self.finish(
            self.base()
                .arr()
                .agg(element().any(ignoreNulls.unwrap_or(true))),
        )
    }

    pub fn all(&self, ignoreNulls: Option<bool>) -> PhpResult<Zval> {
        self.finish(
            self.base()
                .arr()
                .agg(element().all(ignoreNulls.unwrap_or(true))),
        )
    }

    pub fn toList(&self) -> PhpResult<Zval> {
        self.finish(self.base().arr().to_list())
    }

    pub fn sort(&self, descending: Option<bool>, nullsLast: Option<bool>) -> PhpResult<Zval> {
        self.finish(
            self.base().arr().sort(
                SortOptions::default()
                    .with_order_descending(descending.unwrap_or(false))
                    .with_nulls_last(nullsLast.unwrap_or(false)),
            ),
        )
    }

    pub fn reverse(&self) -> PhpResult<Zval> {
        self.finish(self.base().arr().eval(element().reverse(), false))
    }

    pub fn argMin(&self) -> PhpResult<Zval> {
        self.finish(self.base().arr().arg_min())
    }

    pub fn argMax(&self) -> PhpResult<Zval> {
        self.finish(self.base().arr().arg_max())
    }

    pub fn get(&self, index: &Zval, nullOnOob: Option<bool>) -> PhpResult<Zval> {
        self.finish(
            self.base()
                .arr()
                .get(operand(index)?, nullOnOob.unwrap_or(false)),
        )
    }

    pub fn first(&self) -> PhpResult<Zval> {
        self.finish(self.base().arr().get(lit(0i64), true))
    }

    pub fn last(&self) -> PhpResult<Zval> {
        self.finish(self.base().arr().get(lit(-1i64), true))
    }

    pub fn join(&self, separator: &Zval, ignoreNulls: Option<bool>) -> PhpResult<Zval> {
        self.finish(
            self.base()
                .arr()
                .join(operand(separator)?, ignoreNulls.unwrap_or(true)),
        )
    }

    pub fn explode(&self) -> PhpResult<Zval> {
        self.finish(self.base().arr().explode(ExplodeOptions {
            empty_as_null: true,
            keep_nulls: true,
        }))
    }

    pub fn contains(&self, item: &Zval, nullsEqual: Option<bool>) -> PhpResult<Zval> {
        self.finish(
            self.base()
                .arr()
                .contains(operand(item)?, nullsEqual.unwrap_or(true)),
        )
    }

    pub fn countMatches(&self, element: &Zval) -> PhpResult<Zval> {
        self.finish(self.base().arr().count_matches(operand(element)?))
    }

    /// A struct with fields named field_0, field_1, ...
    pub fn toStruct(&self) -> PhpResult<Zval> {
        self.finish(self.base().arr().to_struct(None))
    }

    pub fn shift(&self, n: Option<&Zval>) -> PhpResult<Zval> {
        let n = match crate::input::given(n) {
            Some(n) => operand(n)?,
            None => lit(1i64),
        };
        self.finish(self.base().arr().shift(n))
    }

    pub fn slice(
        &self,
        offset: &Zval,
        length: Option<&Zval>,
        asArray: Option<bool>,
    ) -> PhpResult<Zval> {
        let length = match crate::input::given(length) {
            Some(l) => operand(l)?,
            None => lit(Null {}),
        };
        self.finish(
            self.base()
                .arr()
                .slice(operand(offset)?, length, asArray.unwrap_or(false))
                .php()?,
        )
    }

    pub fn head(&self, n: Option<i64>, asArray: Option<bool>) -> PhpResult<Zval> {
        let n = crate::input::count(n.unwrap_or(5), "n")?;
        self.finish(
            self.base()
                .arr()
                .slice(lit(0i64), lit(n as i64), asArray.unwrap_or(false))
                .php()?,
        )
    }

    pub fn tail(&self, n: Option<i64>, asArray: Option<bool>) -> PhpResult<Zval> {
        let n = crate::input::count(n.unwrap_or(5), "n")? as i64;
        self.finish(
            self.base()
                .arr()
                .slice(lit(-n), lit(n), asArray.unwrap_or(false))
                .php()?,
        )
    }

    /// Run an expression on each array's elements: eval(element()->mul(2)).
    pub fn eval(&self, expr: &Zval, asList: Option<bool>) -> PhpResult<Zval> {
        self.finish(
            self.base()
                .arr()
                .eval(into_expr(expr)?, asList.unwrap_or(false)),
        )
    }

    pub fn agg(&self, expr: &Zval) -> PhpResult<Zval> {
        self.finish(self.base().arr().agg(into_expr(expr)?))
    }
}

#[php_impl]
impl PhpBinNameSpace {
    pub fn contains(&self, literal: &Zval) -> PhpResult<Zval> {
        self.finish(self.base().binary().contains_literal(operand(literal)?))
    }

    pub fn startsWith(&self, prefix: &Zval) -> PhpResult<Zval> {
        self.finish(self.base().binary().starts_with(operand(prefix)?))
    }

    pub fn endsWith(&self, suffix: &Zval) -> PhpResult<Zval> {
        self.finish(self.base().binary().ends_with(operand(suffix)?))
    }

    /// 'hex' or 'base64' text.
    pub fn encode(&self, encoding: String) -> PhpResult<Zval> {
        self.finish(match encoding.as_str() {
            "hex" => self.base().binary().hex_encode(),
            "base64" => self.base().binary().base64_encode(),
            e => {
                return Err(value_error(format!(
                    "encoding must be hex or base64, not '{e}'"
                )));
            }
        })
    }

    pub fn decode(&self, encoding: String, strict: Option<bool>) -> PhpResult<Zval> {
        let strict = strict.unwrap_or(true);
        self.finish(match encoding.as_str() {
            "hex" => self.base().binary().hex_decode(strict),
            "base64" => self.base().binary().base64_decode(strict),
            e => {
                return Err(value_error(format!(
                    "encoding must be hex or base64, not '{e}'"
                )));
            }
        })
    }

    /// The size in 'b', 'kb', 'mb', 'gb', or 'tb'.
    pub fn size(&self, unit: Option<String>) -> PhpResult<Zval> {
        let bytes = self.base().binary().size_bytes();
        let div = match unit.as_deref().unwrap_or("b") {
            "b" | "bytes" => return self.finish(bytes),
            "kb" | "kilobytes" => 1024f64,
            "mb" | "megabytes" => 1024f64.powi(2),
            "gb" | "gigabytes" => 1024f64.powi(3),
            "tb" | "terabytes" => 1024f64.powi(4),
            u => return Err(value_error(format!("unknown size unit '{u}'"))),
        };
        self.finish(bytes.cast(DataType::Float64) / lit(div))
    }

    pub fn slice(&self, offset: &Zval, length: Option<&Zval>) -> PhpResult<Zval> {
        let length = match crate::input::given(length) {
            Some(l) => operand(l)?,
            None => lit(Null {}),
        };
        self.finish(self.base().binary().slice(operand(offset)?, length))
    }

    pub fn head(&self, n: Option<i64>) -> PhpResult<Zval> {
        self.finish(self.base().binary().head(lit(n.unwrap_or(5))))
    }

    pub fn tail(&self, n: Option<i64>) -> PhpResult<Zval> {
        self.finish(self.base().binary().tail(lit(n.unwrap_or(5))))
    }

    pub fn get(&self, index: &Zval, nullOnOob: Option<bool>) -> PhpResult<Zval> {
        self.finish(
            self.base()
                .binary()
                .get(operand(index)?, nullOnOob.unwrap_or(false)),
        )
    }
}
