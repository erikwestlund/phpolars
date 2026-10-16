//! Data types as py-polars names: 'Int64', 'String', 'Datetime(ms, UTC)', 'List(Int64)'.
use ext_php_rs::prelude::*;
use polars::prelude::*;

use crate::error::value_error;

fn time_unit(s: &str) -> PhpResult<TimeUnit> {
    match s.trim() {
        "ns" => Ok(TimeUnit::Nanoseconds),
        "us" | "μs" => Ok(TimeUnit::Microseconds),
        "ms" => Ok(TimeUnit::Milliseconds),
        other => Err(value_error(format!(
            "unknown time unit '{other}'; use ns, us or ms"
        ))),
    }
}

pub fn unit_name(tu: TimeUnit) -> &'static str {
    match tu {
        TimeUnit::Nanoseconds => "ns",
        TimeUnit::Microseconds => "us",
        TimeUnit::Milliseconds => "ms",
    }
}

/// Split "Name(args)" into ("Name", Some("args")).
fn split(s: &str) -> PhpResult<(&str, Option<&str>)> {
    match s.find('(') {
        None => Ok((s, None)),
        Some(i) if s.ends_with(')') => Ok((&s[..i], Some(&s[i + 1..s.len() - 1]))),
        Some(_) => Err(value_error(format!("malformed data type '{s}'"))),
    }
}

/// Split on commas outside parentheses: "a: List(Int64), b: String".
fn split_top(s: &str) -> Vec<&str> {
    let (mut depth, mut start, mut out) = (0i32, 0, Vec::new());
    for (i, c) in s.char_indices() {
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' => depth -= 1,
            ',' if depth == 0 => {
                out.push(s[start..i].trim());
                start = i + 1;
            }
            _ => {}
        }
    }
    out.push(s[start..].trim());
    out
}

pub fn parse(s: &str) -> PhpResult<DataType> {
    let s = s.trim();
    let (name, args) = split(s)?;
    let dtype = match (name, args) {
        ("Boolean" | "Bool", None) => DataType::Boolean,
        ("Int8", None) => DataType::Int8,
        ("Int16", None) => DataType::Int16,
        ("Int32", None) => DataType::Int32,
        ("Int64", None) => DataType::Int64,
        ("UInt8", None) => DataType::UInt8,
        ("UInt16", None) => DataType::UInt16,
        ("UInt32", None) => DataType::UInt32,
        ("UInt64", None) => DataType::UInt64,
        ("Float32", None) => DataType::Float32,
        ("Float64", None) => DataType::Float64,
        ("String" | "Utf8", None) => DataType::String,
        ("Binary", None) => DataType::Binary,
        ("Date", None) => DataType::Date,
        ("Time", None) => DataType::Time,
        ("Null", None) => DataType::Null,
        ("Categorical", None) => DataType::from_categories(Categories::global()),
        ("Datetime", None) => DataType::Datetime(TimeUnit::Microseconds, None),
        ("Datetime", Some(a)) => {
            let mut parts = a.splitn(2, ',');
            let tu = time_unit(parts.next().unwrap_or("us"))?;
            let tz = parts
                .next()
                .map(|t| t.trim().trim_matches('\'').to_string());
            DataType::Datetime(
                tu,
                TimeZone::opt_try_new(tz).map_err(crate::error::polars_error)?,
            )
        }
        ("Duration", None) => DataType::Duration(TimeUnit::Microseconds),
        ("Duration", Some(a)) => DataType::Duration(time_unit(a)?),
        ("Decimal", Some(a)) => {
            let mut parts = a.split(',').map(|p| p.trim().parse::<usize>());
            match (parts.next(), parts.next()) {
                (Some(Ok(p)), Some(Ok(sc))) => DataType::Decimal(p, sc),
                _ => return Err(value_error(format!("malformed data type '{s}'"))),
            }
        }
        ("List", Some(inner)) => DataType::List(Box::new(parse(inner)?)),
        ("Int128", None) => DataType::Int128,
        ("UInt128", None) => DataType::UInt128,
        ("Array", Some(a)) => match split_top(a).as_slice() {
            [inner, n] => DataType::Array(
                Box::new(parse(inner)?),
                n.parse::<usize>()
                    .map_err(|_| value_error(format!("malformed data type '{s}'")))?,
            ),
            _ => return Err(value_error(format!("malformed data type '{s}'"))),
        },
        ("Struct", Some(a)) => {
            let mut fields = Vec::new();
            for f in split_top(a).into_iter().filter(|f| !f.is_empty()) {
                let (n, t) = f
                    .split_once(':')
                    .ok_or_else(|| value_error(format!("malformed struct field '{f}'")))?;
                fields.push(Field::new(n.trim().into(), parse(t)?));
            }
            DataType::Struct(fields)
        }
        _ => return Err(value_error(format!("unknown data type '{s}'"))),
    };
    Ok(dtype)
}

pub fn name(dtype: &DataType) -> String {
    match dtype {
        DataType::Boolean => "Boolean".into(),
        DataType::UInt8 => "UInt8".into(),
        DataType::UInt16 => "UInt16".into(),
        DataType::UInt32 => "UInt32".into(),
        DataType::UInt64 => "UInt64".into(),
        DataType::UInt128 => "UInt128".into(),
        DataType::Int8 => "Int8".into(),
        DataType::Int16 => "Int16".into(),
        DataType::Int32 => "Int32".into(),
        DataType::Int64 => "Int64".into(),
        DataType::Int128 => "Int128".into(),
        DataType::Float16 => "Float16".into(),
        DataType::Float32 => "Float32".into(),
        DataType::Float64 => "Float64".into(),
        DataType::Decimal(p, s) => format!("Decimal({p}, {s})"),
        DataType::String => "String".into(),
        DataType::Binary | DataType::BinaryOffset => "Binary".into(),
        DataType::Date => "Date".into(),
        DataType::Datetime(tu, None) => format!("Datetime({})", unit_name(*tu)),
        DataType::Datetime(tu, Some(tz)) => {
            format!("Datetime({}, {})", unit_name(*tu), tz.as_str())
        }
        DataType::Duration(tu) => format!("Duration({})", unit_name(*tu)),
        DataType::Time => "Time".into(),
        DataType::Array(inner, n) => format!("Array({}, {n})", name(inner)),
        DataType::List(inner) => format!("List({})", name(inner)),
        DataType::Null => "Null".into(),
        DataType::Categorical(..) => "Categorical".into(),
        DataType::Enum(..) => "Enum".into(),
        DataType::Struct(fields) => format!(
            "Struct({})",
            fields
                .iter()
                .map(|f| format!("{}: {}", f.name(), name(f.dtype())))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        other => format!("{other:?}"),
    }
}

pub fn schema_array(schema: &Schema) -> Vec<(String, String)> {
    schema
        .iter()
        .map(|(n, dt)| (n.to_string(), name(dt)))
        .collect()
}

/// Constants for IDE completion; the values are the strings every method accepts.
#[php_class]
#[php(name = "Phpolars\\DataType")]
pub struct PhpDataType;

#[php_impl]
#[php(change_constant_case = "none")]
#[allow(non_upper_case_globals)]
impl PhpDataType {
    pub const Boolean: &'static str = "Boolean";
    pub const Int8: &'static str = "Int8";
    pub const Int16: &'static str = "Int16";
    pub const Int32: &'static str = "Int32";
    pub const Int64: &'static str = "Int64";
    pub const UInt8: &'static str = "UInt8";
    pub const UInt16: &'static str = "UInt16";
    pub const UInt32: &'static str = "UInt32";
    pub const UInt64: &'static str = "UInt64";
    pub const Float32: &'static str = "Float32";
    pub const Float64: &'static str = "Float64";
    pub const String: &'static str = "String";
    pub const Binary: &'static str = "Binary";
    pub const Date: &'static str = "Date";
    pub const Datetime: &'static str = "Datetime";
    pub const Duration: &'static str = "Duration";
    pub const Time: &'static str = "Time";
    pub const Categorical: &'static str = "Categorical";
    pub const Null: &'static str = "Null";

    /// Datetime with a unit and optional time zone, e.g. datetime('ms', 'UTC').
    pub fn datetime(unit: Option<String>, timeZone: Option<String>) -> String {
        let unit = unit.unwrap_or_else(|| "us".into());
        match timeZone {
            Some(tz) => format!("Datetime({unit}, {tz})"),
            None => format!("Datetime({unit})"),
        }
    }

    pub fn duration(unit: Option<String>) -> String {
        format!("Duration({})", unit.unwrap_or_else(|| "us".into()))
    }

    pub fn list(inner: String) -> String {
        format!("List({inner})")
    }
}
