//! Moving data between PHP values and Polars. NA is PHP null.
//!
//! Polars → PHP: integers → int, floats → float, Boolean → bool, String and
//! Categorical → string, Date/Datetime → DateTimeImmutable (naive values in
//! UTC), Time → "HH:MM:SS.ffffff", Duration → int in its time unit, Decimal →
//! string, Binary → string, List/Array → list, Struct → associative array.
//!
//! PHP → Polars infers one type per column: ints widen to floats, nulls fit
//! anything, DateTimeInterface → Datetime(us) on its wall clock, lists → List,
//! associative arrays → Struct. Other mixes are errors, not silent casts.
use chrono::{DateTime, NaiveDateTime};
use ext_php_rs::{
    boxed::ZBox,
    prelude::*,
    types::{ZendCallable, ZendHashTable, Zval},
    zend::ClassEntry,
};
use polars::prelude::*;

use crate::{
    error::{OrPhp, value_error},
    expr::PhpExpr,
    series::PhpSeries,
};

fn null() -> Zval {
    let mut z = Zval::new();
    z.set_null();
    z
}

fn values<T>(it: impl Iterator<Item = Option<T>>, set: impl Fn(&mut Zval, T)) -> Vec<Zval> {
    it.map(|v| match v {
        Some(v) => {
            let mut z = Zval::new();
            set(&mut z, v);
            z
        }
        None => null(),
    })
    .collect()
}

fn call(function: &str, args: Vec<&dyn ext_php_rs::convert::IntoZvalDyn>) -> PhpResult<Zval> {
    let f = ZendCallable::try_from_name(function)
        .map_err(|_| PhpException::from(format!("{function}() is not available")))?;
    Ok(f.try_call(args)?)
}

/// A DateTimeImmutable for a UTC instant, moved into `tz` when given.
/// Makes DateTimeImmutable values for one column. The PHP function and the
/// time zones are looked up once, not per value.
struct DateMaker {
    create: ZendCallable<'static>,
    utc: Zval,
    zone: Option<Zval>,
}

impl DateMaker {
    fn new(tz: Option<&str>) -> PhpResult<Self> {
        let create = ZendCallable::try_from_name("date_create_immutable")
            .map_err(|_| PhpException::from("date_create_immutable() is not available"))?;
        let utc = call("timezone_open", vec![&"UTC"])?;
        let zone = match tz {
            Some(tz) => Some(call("timezone_open", vec![&tz])?),
            None => None,
        };
        Ok(Self { create, utc, zone })
    }

    /// A UTC instant, moved into the column's zone when it has one.
    fn make(&self, dt: NaiveDateTime) -> PhpResult<Zval> {
        use std::fmt::Write;
        let mut text = String::with_capacity(26);
        let _ = write!(text, "{}", dt.format("%Y-%m-%d %H:%M:%S%.6f"));
        let value = self.create.try_call(vec![&text, &self.utc])?;
        match &self.zone {
            None => Ok(value),
            Some(zone) => Ok(value.try_call_method("setTimezone", vec![zone])?),
        }
    }

    fn column(
        &self,
        values: impl Iterator<Item = Option<i64>>,
        tu: TimeUnit,
    ) -> PhpResult<Vec<Zval>> {
        values
            .map(|v| match v.and_then(|v| naive(v, tu)) {
                Some(dt) => self.make(dt),
                None => Ok(null()),
            })
            .collect()
    }
}

fn naive(value: i64, tu: TimeUnit) -> Option<NaiveDateTime> {
    let dt = match tu {
        TimeUnit::Nanoseconds => DateTime::from_timestamp_nanos(value),
        TimeUnit::Microseconds => DateTime::from_timestamp_micros(value)?,
        TimeUnit::Milliseconds => DateTime::from_timestamp_millis(value)?,
    };
    Some(dt.naive_utc())
}

fn time_string(nanos: i64) -> String {
    let secs = nanos / 1_000_000_000;
    let frac = (nanos % 1_000_000_000) / 1_000;
    format!(
        "{:02}:{:02}:{:02}.{:06}",
        secs / 3600,
        secs / 60 % 60,
        secs % 60,
        frac
    )
}

fn hashtable(z: ZBox<ZendHashTable>) -> Zval {
    let mut out = Zval::new();
    out.set_hashtable(z);
    out
}

/// One Zval per element of the series.
pub fn series_values(s: &Series) -> PhpResult<Vec<Zval>> {
    let s = s.rechunk();
    Ok(match s.dtype() {
        DataType::Null => (0..s.len()).map(|_| null()).collect(),
        DataType::Boolean => values(s.bool().php()?.iter(), |z, v| z.set_bool(v)),
        DataType::String => values(s.str().php()?.iter(), |z, v| {
            let _ = z.set_string(v, false);
        }),
        DataType::Float16 | DataType::Float32 | DataType::Float64 => {
            let s = s.cast(&DataType::Float64).php()?;
            values(s.f64().php()?.iter(), |z, v| z.set_double(v))
        }
        DataType::UInt64 | DataType::UInt128 | DataType::Int128 => {
            // Values beyond int64 would wrap; return those columns as floats, as PHP does.
            match s.strict_cast(&DataType::Int64) {
                Ok(n) => values(n.i64().php()?.iter(), |z, v| z.set_long(v)),
                Err(_) => {
                    let wide = s.cast(&DataType::Float64).php()?;
                    values(wide.f64().php()?.iter(), |z, v| z.set_double(v))
                }
            }
        }
        dt if dt.is_integer() => {
            let s = s.cast(&DataType::Int64).php()?;
            values(s.i64().php()?.iter(), |z, v| z.set_long(v))
        }
        DataType::Date => {
            let days = s.to_physical_repr();
            let millis = days
                .i32()
                .php()?
                .iter()
                .map(|v| v.map(|d| d as i64 * 86_400_000));
            DateMaker::new(None)?.column(millis, TimeUnit::Milliseconds)?
        }
        DataType::Datetime(tu, tz) => {
            let phys = s.to_physical_repr();
            DateMaker::new(tz.as_ref().map(|t| t.as_str()))?
                .column(phys.i64().php()?.iter(), *tu)?
        }
        DataType::Duration(_) => {
            let phys = s.to_physical_repr();
            values(phys.i64().php()?.iter(), |z, v| z.set_long(v))
        }
        DataType::Time => {
            let phys = s.to_physical_repr();
            values(phys.i64().php()?.iter(), |z, v| {
                let _ = z.set_string(&time_string(v), false);
            })
        }
        DataType::Binary => values(s.binary().php()?.iter(), |z, v| z.set_binary(v.to_vec())),
        DataType::List(_) => {
            let ca = s.list().php()?;
            let mut out = Vec::with_capacity(s.len());
            for i in 0..ca.len() {
                out.push(match ca.get_as_series(i) {
                    Some(inner) => hashtable(series_list(&inner)?),
                    None => null(),
                });
            }
            out
        }
        DataType::Array(inner, _) => {
            let as_list = s.cast(&DataType::List(inner.clone())).php()?;
            series_values(&as_list)?
        }
        DataType::Struct(_) => {
            let ca = s.struct_().php()?;
            let mut columns = Vec::new();
            for f in ca.fields_as_series() {
                columns.push((f.name().to_string(), series_values(&f)?.into_iter()));
            }
            let mut out = Vec::with_capacity(s.len());
            for valid in s.is_not_null().iter() {
                let mut row = ZendHashTable::with_capacity(columns.len() as u32);
                for (name, it) in columns.iter_mut() {
                    row.insert(name.as_str(), it.next().unwrap_or_else(null))?;
                }
                out.push(if valid == Some(true) {
                    hashtable(row)
                } else {
                    null()
                });
            }
            out
        }
        _ => {
            let s = s.cast(&DataType::String).php()?;
            values(s.str().php()?.iter(), |z, v| {
                let _ = z.set_string(v, false);
            })
        }
    })
}

pub fn series_list(s: &Series) -> PhpResult<ZBox<ZendHashTable>> {
    let vals = series_values(s)?;
    let mut ht = ZendHashTable::with_capacity(vals.len() as u32);
    for v in vals {
        ht.push(v)?;
    }
    Ok(ht)
}

/// The first value of a series, or null when it is empty.
pub fn scalar(s: &Series) -> PhpResult<Zval> {
    Ok(series_values(&s.head(Some(1)))?.pop().unwrap_or_else(null))
}

pub fn to_rows(df: &DataFrame) -> PhpResult<ZBox<ZendHashTable>> {
    let mut rows: Vec<ZBox<ZendHashTable>> = (0..df.height())
        .map(|_| ZendHashTable::with_capacity(df.width() as u32))
        .collect();
    for column in df.columns() {
        let name = column.name().as_str();
        let vals = series_values(column.as_materialized_series())?;
        for (row, v) in rows.iter_mut().zip(vals) {
            row.insert(name, v)?;
        }
    }
    let mut out = ZendHashTable::with_capacity(rows.len() as u32);
    for row in rows {
        out.push(row)?;
    }
    Ok(out)
}

/// Rows as lists of values, like py-polars rows().
pub fn to_tuples(df: &DataFrame) -> PhpResult<ZBox<ZendHashTable>> {
    let mut rows: Vec<ZBox<ZendHashTable>> = (0..df.height())
        .map(|_| ZendHashTable::with_capacity(df.width() as u32))
        .collect();
    for column in df.columns() {
        let vals = series_values(column.as_materialized_series())?;
        for (row, v) in rows.iter_mut().zip(vals) {
            row.push(v)?;
        }
    }
    let mut out = ZendHashTable::with_capacity(rows.len() as u32);
    for row in rows {
        out.push(row)?;
    }
    Ok(out)
}

pub fn to_columns(df: &DataFrame) -> PhpResult<ZBox<ZendHashTable>> {
    let mut out = ZendHashTable::with_capacity(df.width() as u32);
    for column in df.columns() {
        out.insert(
            column.name().as_str(),
            series_list(column.as_materialized_series())?,
        )?;
    }
    Ok(out)
}

// PHP → Polars

#[derive(Clone, Copy, PartialEq, Debug)]
enum Kind {
    Null,
    Bool,
    Int,
    Float,
    Str,
    Other,
}

fn kind(v: &Zval) -> Kind {
    if v.is_null() {
        Kind::Null
    } else if v.is_bool() {
        Kind::Bool
    } else if v.is_long() {
        Kind::Int
    } else if v.is_double() {
        Kind::Float
    } else if v.is_string() {
        Kind::Str
    } else {
        Kind::Other
    }
}

pub fn is_list(ht: &ZendHashTable) -> bool {
    ht.has_sequential_keys()
}

fn is_datetime(v: &Zval) -> bool {
    match (v.object(), ClassEntry::try_find("DateTimeInterface")) {
        (Some(obj), Some(ce)) => obj.instance_of(ce),
        _ => false,
    }
}

/// Microseconds since the epoch on the value's wall clock.
fn wall_clock_micros(v: &Zval) -> PhpResult<i64> {
    let text = v.try_call_method("format", vec![&"Y-m-d H:i:s.u"])?;
    let text = text.str().unwrap_or_default();
    if let Some(micros) = fast_wall_clock(text) {
        return Ok(micros);
    }
    let dt = NaiveDateTime::parse_from_str(text, "%Y-%m-%d %H:%M:%S%.f")
        .map_err(|e| value_error(format!("cannot read date '{text}': {e}")))?;
    Ok(dt.and_utc().timestamp_micros())
}

/// "YYYY-mm-dd HH:MM:SS.uuuuuu" with a four-digit year, read without chrono's
/// format parser. Other shapes return None and take the general path.
fn fast_wall_clock(t: &str) -> Option<i64> {
    let b = t.as_bytes();
    if b.len() != 26
        || b[4] != b'-'
        || b[7] != b'-'
        || b[10] != b' '
        || b[13] != b':'
        || b[16] != b':'
        || b[19] != b'.'
    {
        return None;
    }
    let num = |r: std::ops::Range<usize>| -> Option<u32> { t.get(r)?.parse().ok() };
    let date = chrono::NaiveDate::from_ymd_opt(num(0..4)? as i32, num(5..7)?, num(8..10)?)?;
    let time = chrono::NaiveTime::from_hms_micro_opt(
        num(11..13)?,
        num(14..16)?,
        num(17..19)?,
        num(20..26)?,
    )?;
    Some(NaiveDateTime::new(date, time).and_utc().timestamp_micros())
}

/// A column of DateTimeInterface values (and nulls) read straight into
/// Datetime(us), or None when anything else is in it.
fn datetime_column(name: PlSmallStr, vals: &[&Zval]) -> PhpResult<Option<Series>> {
    let Some(ce) = ClassEntry::try_find("DateTimeInterface") else {
        return Ok(None);
    };
    let mut out = Vec::with_capacity(vals.len());
    for v in vals {
        if v.is_null() {
            out.push(None);
        } else if v.object().is_some_and(|o| o.instance_of(ce)) {
            out.push(Some(wall_clock_micros(v)?));
        } else {
            return Ok(None);
        }
    }
    let s = Series::new(name, out);
    Ok(Some(
        s.cast(&DataType::Datetime(TimeUnit::Microseconds, None))
            .php()?,
    ))
}

/// The kind of each leaf value per position (list elements, struct fields).
fn leaf_kinds(av: &AnyValue, path: &mut String, out: &mut Vec<(String, &'static str)>) {
    let kind = match av {
        AnyValue::Null => return,
        AnyValue::List(s) => {
            path.push_str("[]");
            for inner in s.iter() {
                leaf_kinds(&inner.into_static(), path, out);
            }
            path.truncate(path.len() - 2);
            return;
        }
        AnyValue::StructOwned(payload) => {
            let (vals, fields) = payload.as_ref();
            for (v, f) in vals.iter().zip(fields) {
                let len = path.len();
                path.push('.');
                path.push_str(f.name());
                leaf_kinds(v, path, out);
                path.truncate(len);
            }
            return;
        }
        AnyValue::Boolean(_) => "bool",
        AnyValue::StringOwned(_) | AnyValue::String(_) => "string",
        AnyValue::DatetimeOwned(..) | AnyValue::Datetime(..) | AnyValue::Date(_) => "datetime",
        v if v.is_primitive_numeric() => "number",
        _ => "other",
    };
    if !out.iter().any(|(p, k)| p == path && *k == kind) {
        out.push((path.clone(), kind));
    }
}

/// Mixed kinds at one position (a list holding ints and strings, a struct
/// field holding both) raise SchemaError instead of becoming strings.
fn check_kinds(name: &str, avs: &[AnyValue]) -> PhpResult<()> {
    let mut kinds = Vec::new();
    for av in avs {
        leaf_kinds(av, &mut String::new(), &mut kinds);
    }
    for (i, (path, kind)) in kinds.iter().enumerate() {
        if let Some((_, other)) = kinds[i + 1..].iter().find(|(p, k)| p == path && k != kind) {
            return Err(PhpException::from_class::<crate::error::SchemaError>(
                format!(
                    "column '{name}' mixes {kind} and {other} values{}; pass a dtype or cast first",
                    if path.is_empty() {
                        String::new()
                    } else {
                        format!(" at {path}")
                    }
                ),
            ));
        }
    }
    Ok(())
}

/// A PHP value as an owned AnyValue, for nested and temporal data.
pub fn any_value(v: &Zval) -> PhpResult<AnyValue<'static>> {
    Ok(match kind(v) {
        Kind::Null => AnyValue::Null,
        Kind::Bool => AnyValue::Boolean(v.bool().unwrap()),
        Kind::Int => AnyValue::Int64(v.long().unwrap()),
        Kind::Float => AnyValue::Float64(v.double().unwrap()),
        Kind::Str => AnyValue::StringOwned(v.str().unwrap().into()),
        Kind::Other => {
            if let Some(ht) = v.array() {
                if is_list(ht) {
                    AnyValue::List(series_from_array("", ht, None)?)
                } else {
                    let mut vals = Vec::with_capacity(ht.len());
                    let mut fields = Vec::with_capacity(ht.len());
                    for (k, item) in ht.iter() {
                        let av = any_value(item)?;
                        fields.push(Field::new(k.to_string().into(), av.dtype()));
                        vals.push(av);
                    }
                    AnyValue::StructOwned(Box::new((vals, fields)))
                }
            } else if is_datetime(v) {
                AnyValue::DatetimeOwned(wall_clock_micros(v)?, TimeUnit::Microseconds, None)
            } else if let Some(s) = v.extract::<&PhpSeries>() {
                AnyValue::List(s.inner.clone())
            } else {
                return Err(value_error(format!(
                    "cannot convert a {} to a Polars value",
                    v.get_type()
                )));
            }
        }
    })
}

/// Build a series from PHP values, inferring the type unless `dtype` is given.
pub fn series(name: PlSmallStr, vals: &[&Zval], dtype: Option<&DataType>) -> PhpResult<Series> {
    let mut k = Kind::Null;
    for v in vals {
        k = match (k, kind(v)) {
            (a, Kind::Null) => a,
            (Kind::Null, b) => b,
            (Kind::Int, Kind::Float) | (Kind::Float, Kind::Int) => Kind::Float,
            (a, b) if a == b => a,
            (Kind::Other, _) | (_, Kind::Other) => Kind::Other,
            _ => {
                return Err(PhpException::from_class::<crate::error::SchemaError>(
                    format!("column '{name}' mixes incompatible types; pass a dtype or cast first"),
                ));
            }
        };
    }
    let s = match k {
        Kind::Null => Series::full_null(name, vals.len(), &DataType::Null),
        Kind::Bool => Series::new(name, vals.iter().map(|v| v.bool()).collect::<Vec<_>>()),
        Kind::Int => Series::new(name, vals.iter().map(|v| v.long()).collect::<Vec<_>>()),
        Kind::Float => Series::new(
            name,
            vals.iter()
                .map(|v| v.double().or_else(|| v.long().map(|i| i as f64)))
                .collect::<Vec<_>>(),
        ),
        Kind::Str => Series::new(name, vals.iter().map(|v| v.str()).collect::<Vec<_>>()),
        Kind::Other => {
            if let Some(s) = datetime_column(name.clone(), vals)? {
                return match dtype {
                    Some(dt) if dt != s.dtype() => s.strict_cast(dt).php(),
                    _ => Ok(s),
                };
            }
            let avs = vals
                .iter()
                .map(|v| any_value(v))
                .collect::<PhpResult<Vec<_>>>()?;
            // Polars would cast mixed kinds (ints and strings in one list) to strings.
            check_kinds(&name, &avs)?;
            let s = Series::from_any_values(name, &avs, false).php()?;
            return match dtype {
                Some(dt) if dt != s.dtype() => s.strict_cast(dt).php(),
                _ => Ok(s),
            };
        }
    };
    match dtype {
        Some(dt) if dt != s.dtype() => s.strict_cast(dt).php(),
        _ => Ok(s),
    }
}

pub fn series_from_array(
    name: &str,
    list: &ZendHashTable,
    dtype: Option<&DataType>,
) -> PhpResult<Series> {
    let vals: Vec<&Zval> = list.values().collect();
    series(name.into(), &vals, dtype)
}

/// ['name' => 'Int64', ...] → names and parsed types.
pub fn schema_overrides(schema: Option<&ZendHashTable>) -> PhpResult<Vec<(String, DataType)>> {
    let mut out = Vec::new();
    if let Some(schema) = schema {
        for (k, v) in schema.iter() {
            let name = v
                .str()
                .ok_or_else(|| value_error("schema values must be data type names"))?;
            out.push((k.to_string(), crate::dtype::parse(name)?));
        }
    }
    Ok(out)
}

fn lookup<'a>(schema: &'a [(String, DataType)], name: &str) -> Option<&'a DataType> {
    schema.iter().find(|(n, _)| n == name).map(|(_, d)| d)
}

/// ['name' => [values...] | Series, ...]
pub fn from_columns(
    columns: &ZendHashTable,
    schema: Option<&ZendHashTable>,
) -> PhpResult<DataFrame> {
    let schema = schema_overrides(schema)?;
    let mut out: Vec<Column> = Vec::with_capacity(columns.len());
    for (key, value) in columns.iter() {
        let name = key.to_string();
        let s = if let Some(s) = value.extract::<&PhpSeries>() {
            let s = s.inner.clone().with_name(name.as_str().into());
            match lookup(&schema, &name) {
                Some(dt) => s.strict_cast(dt).php()?,
                None => s,
            }
        } else if let Some(list) = value.array() {
            series_from_array(&name, list, lookup(&schema, &name))?
        } else {
            return Err(value_error(format!(
                "column '{name}' must be a list or a Series"
            )));
        };
        out.push(s.into());
    }
    if out.is_empty() {
        let columns = schema
            .iter()
            .map(|(n, dt)| Series::new_empty(n.as_str().into(), dt).into())
            .collect();
        return DataFrame::new_infer_height(columns).php();
    }
    DataFrame::new_infer_height(out).php()
}

/// [['name' => value, ...], ...] (keys from the first row) or [[v1, v2], ...]
/// with names from the schema.
pub fn from_rows(rows: &ZendHashTable, schema: Option<&ZendHashTable>) -> PhpResult<DataFrame> {
    let overrides = schema_overrides(schema)?;
    let mut names: Vec<String> = Vec::new();
    let mut lists: Vec<Vec<&Zval>> = Vec::new();
    let mut positional = false;
    for (i, (_, row)) in rows.iter().enumerate() {
        let row = row
            .array()
            .ok_or_else(|| value_error("each row must be an array"))?;
        if i == 0 {
            positional = is_list(row) && !row.is_empty();
            if positional {
                if overrides.len() != row.len() {
                    return Err(value_error(
                        "rows without keys need a schema naming every column",
                    ));
                }
                names = overrides.iter().map(|(n, _)| n.clone()).collect();
            } else {
                names = row.iter().map(|(k, _)| k.to_string()).collect();
            }
            lists = names
                .iter()
                .map(|_| Vec::with_capacity(rows.len()))
                .collect();
        }
        if positional {
            if row.len() != names.len() {
                return Err(value_error(format!(
                    "row {i} has {} values, expected {}",
                    row.len(),
                    names.len()
                )));
            }
            for (list, v) in lists.iter_mut().zip(row.values()) {
                list.push(v);
            }
        } else {
            for (name, list) in names.iter().zip(lists.iter_mut()) {
                let v = row
                    .get(name.as_str())
                    .ok_or_else(|| value_error(format!("row {i} is missing '{name}'")))?;
                list.push(v);
            }
        }
    }
    if names.is_empty() {
        let columns = overrides
            .iter()
            .map(|(n, dt)| Series::new_empty(n.as_str().into(), dt).into())
            .collect();
        return DataFrame::new_infer_height(columns).php();
    }
    let mut columns: Vec<Column> = Vec::with_capacity(names.len());
    for (name, list) in names.iter().zip(lists.iter()) {
        columns.push(series(name.as_str().into(), list, lookup(&overrides, name))?.into());
    }
    DataFrame::new_infer_height(columns).php()
}

/// A literal expression from a PHP value; strings are string literals.
pub fn literal(value: &Zval) -> PhpResult<Expr> {
    match kind(value) {
        Kind::Null => Ok(lit(Null {})),
        Kind::Bool => Ok(lit(value.bool().unwrap())),
        Kind::Int => Ok(lit(value.long().unwrap())),
        Kind::Float => Ok(lit(value.double().unwrap())),
        Kind::Str => Ok(lit(value.str().unwrap().to_owned())),
        Kind::Other => {
            if let Some(e) = value.extract::<&PhpExpr>() {
                return Ok(e.inner.clone());
            }
            if let Some(s) = value.extract::<&PhpSeries>() {
                return Ok(lit(s.inner.clone()));
            }
            if let Some(ht) = value.array() {
                // A PHP list literal becomes a Series literal, as for isIn().
                return Ok(lit(series_from_array("literal", ht, None)?));
            }
            if is_datetime(value) {
                return Ok(lit(wall_clock_micros(value)?)
                    .cast(DataType::Datetime(TimeUnit::Microseconds, None)));
            }
            Err(value_error(format!(
                "cannot use a {} as a literal",
                value.get_type()
            )))
        }
    }
}
