//! Moving data between PHP arrays and Polars columns. NA is PHP null.
use ext_php_rs::{
    boxed::ZBox,
    prelude::*,
    types::{ZendHashTable, Zval},
};
use polars::prelude::*;

use crate::polars_error;

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

/// One Zval per row. Integers come back as int, floats as float, booleans as
/// bool and strings as string; other types are formatted as strings.
fn column_values(column: &Column) -> PolarsResult<Vec<Zval>> {
    let s = column.as_materialized_series();
    Ok(match s.dtype() {
        DataType::Null => (0..s.len()).map(|_| null()).collect(),
        DataType::Boolean => values(s.bool()?.iter(), |z, v| z.set_bool(v)),
        DataType::String => values(s.str()?.iter(), |z, v| {
            let _ = z.set_string(v, false);
        }),
        DataType::Float32 | DataType::Float64 => {
            let s = s.cast(&DataType::Float64)?;
            values(s.f64()?.iter(), |z, v| z.set_double(v))
        }
        dt if dt.is_integer() => {
            let s = s.cast(&DataType::Int64)?;
            values(s.i64()?.iter(), |z, v| z.set_long(v))
        }
        _ => {
            let s = s.cast(&DataType::String)?;
            values(s.str()?.iter(), |z, v| {
                let _ = z.set_string(v, false);
            })
        }
    })
}

pub fn to_rows(df: &DataFrame) -> PhpResult<ZBox<ZendHashTable>> {
    let mut rows: Vec<ZBox<ZendHashTable>> = (0..df.height())
        .map(|_| ZendHashTable::with_capacity(df.width() as u32))
        .collect();
    for column in df.columns() {
        let name = column.name().as_str();
        let vals = column_values(column).map_err(polars_error)?;
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

pub fn to_columns(df: &DataFrame) -> PhpResult<ZBox<ZendHashTable>> {
    let mut out = ZendHashTable::with_capacity(df.width() as u32);
    for column in df.columns() {
        let mut list = ZendHashTable::with_capacity(df.height() as u32);
        for v in column_values(column).map_err(polars_error)? {
            list.push(v)?;
        }
        out.insert(column.name().as_str(), list)?;
    }
    Ok(out)
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Null,
    Bool,
    Int,
    Float,
    Str,
}

fn kind(v: &Zval) -> PhpResult<Kind> {
    if v.is_null() {
        Ok(Kind::Null)
    } else if v.is_bool() {
        Ok(Kind::Bool)
    } else if v.is_long() {
        Ok(Kind::Int)
    } else if v.is_double() {
        Ok(Kind::Float)
    } else if v.is_string() {
        Ok(Kind::Str)
    } else {
        Err("column values must be null, bool, int, float or string".into())
    }
}

/// Infer one type per column: ints widen to floats, nulls fit anything,
/// any other mix is an error rather than a silent cast.
fn to_column(name: &str, list: &ZendHashTable) -> PhpResult<Column> {
    let vals: Vec<&Zval> = list.values().collect();
    let mut k = Kind::Null;
    for v in &vals {
        k = match (k, kind(v)?) {
            (a, Kind::Null) => a,
            (Kind::Null, b) => b,
            (Kind::Int, Kind::Float) | (Kind::Float, Kind::Int) => Kind::Float,
            (a, b) if a == b => a,
            _ => return Err(format!("column '{name}' mixes incompatible types").into()),
        };
    }
    let s = match k {
        Kind::Null => Series::full_null(name.into(), vals.len(), &DataType::Null),
        Kind::Bool => Series::new(name.into(), vals.iter().map(|v| v.bool()).collect::<Vec<_>>()),
        Kind::Int => Series::new(name.into(), vals.iter().map(|v| v.long()).collect::<Vec<_>>()),
        Kind::Float => Series::new(
            name.into(),
            vals.iter()
                .map(|v| v.double().or_else(|| v.long().map(|i| i as f64)))
                .collect::<Vec<_>>(),
        ),
        Kind::Str => Series::new(
            name.into(),
            vals.iter().map(|v| v.str()).collect::<Vec<_>>(),
        ),
    };
    Ok(s.into())
}

/// Build a frame from ['name' => [values...], ...].
pub fn from_columns(columns: &ZendHashTable) -> PhpResult<DataFrame> {
    let mut out = Vec::with_capacity(columns.len());
    for (key, list) in columns.iter() {
        let name = key.to_string();
        let list = list
            .array()
            .ok_or_else(|| PhpException::from(format!("column '{name}' must be a list")))?;
        out.push(to_column(&name, list)?);
    }
    DataFrame::new_infer_height(out).map_err(polars_error)
}

/// Build a frame from a list of associative rows sharing the first row's keys.
pub fn from_rows(rows: &ZendHashTable) -> PhpResult<DataFrame> {
    let mut names: Vec<String> = Vec::new();
    let mut lists: Vec<ZBox<ZendHashTable>> = Vec::new();
    for (i, (_, row)) in rows.iter().enumerate() {
        let row = row
            .array()
            .ok_or_else(|| PhpException::from("each row must be an array"))?;
        if i == 0 {
            for (key, _) in row.iter() {
                names.push(key.to_string());
                lists.push(ZendHashTable::new());
            }
        }
        for (name, list) in names.iter().zip(lists.iter_mut()) {
            let v = row
                .get(name.as_str())
                .ok_or_else(|| PhpException::from(format!("row {i} is missing '{name}'")))?;
            list.push(v.shallow_clone())?;
        }
    }
    let mut columns = Vec::with_capacity(names.len());
    for (name, list) in names.iter().zip(lists.iter()) {
        columns.push(to_column(name, list)?);
    }
    DataFrame::new_infer_height(columns).map_err(polars_error)
}
