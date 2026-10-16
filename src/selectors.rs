//! Column selectors, as py's `polars.selectors` (imported there as `cs`):
//! `Phpolars\cs\numeric()`, `cs\starts_with('x')`. A selector is an Expr, and
//! or(), and(), xor(), sub(), and not() combine selectors as sets. PHP reserves
//! list, array, and empty, so those selectors are list_(), array_(), and
//! empty_(), as struct_() and enum_() are.
use ext_php_rs::{prelude::*, types::Zval};
use polars::prelude::*;

use crate::{
    error::value_error,
    expr::{PhpExpr, wrap},
    input::names,
};

fn sel(s: Selector) -> PhpExpr {
    wrap(s.as_expr())
}

fn dtypes(d: DataTypeSelector) -> PhpExpr {
    sel(Selector::ByDType(d))
}

fn of(ds: &[DataType]) -> PhpExpr {
    dtypes(DataTypeSelector::AnyOf(ds.into()))
}

/// Escape a literal for use inside a regex.
fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if "\\.+*?()|[]{}^$#&-~".contains(c) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

fn any_of_pattern(parts: Vec<String>, prefix: &str, suffix: &str) -> PhpExpr {
    let alt = parts
        .iter()
        .map(|p| escape(p))
        .collect::<Vec<_>>()
        .join("|");
    sel(Selector::Matches(format!("{prefix}({alt}){suffix}").into()))
}

/// The selector inside an Expr. With `columns`, a col('x') Expr counts as
/// by_name('x'), as py's selector |, &, and ^ treat it.
pub fn as_selector(v: &Zval, columns: bool) -> Option<Selector> {
    match &v.extract::<&PhpExpr>()?.inner {
        Expr::Selector(s) => Some(s.clone()),
        Expr::Column(n) if columns => Some(by_name([n.clone()], true, false)),
        _ => None,
    }
}

macro_rules! dtype_selector {
    ($php:literal, $rust:ident, $body:expr) => {
        #[php_function]
        #[php(name = $php)]
        pub fn $rust() -> PhpExpr {
            $body
        }
    };
}

dtype_selector!(
    "Phpolars\\cs\\numeric",
    numeric,
    dtypes(DataTypeSelector::Numeric)
);
dtype_selector!(
    "Phpolars\\cs\\integer",
    integer,
    dtypes(DataTypeSelector::Integer)
);
dtype_selector!(
    "Phpolars\\cs\\signed_integer",
    signed_integer,
    dtypes(DataTypeSelector::SignedInteger)
);
dtype_selector!(
    "Phpolars\\cs\\unsigned_integer",
    unsigned_integer,
    dtypes(DataTypeSelector::UnsignedInteger)
);
dtype_selector!(
    "Phpolars\\cs\\float",
    float,
    dtypes(DataTypeSelector::Float)
);
dtype_selector!(
    "Phpolars\\cs\\decimal",
    decimal,
    dtypes(DataTypeSelector::Decimal)
);
dtype_selector!(
    "Phpolars\\cs\\temporal",
    temporal,
    dtypes(DataTypeSelector::Temporal)
);
dtype_selector!(
    "Phpolars\\cs\\categorical",
    categorical,
    dtypes(DataTypeSelector::Categorical)
);
dtype_selector!("Phpolars\\cs\\enum_", enum_, dtypes(DataTypeSelector::Enum));
dtype_selector!(
    "Phpolars\\cs\\nested",
    nested,
    dtypes(DataTypeSelector::Nested)
);
dtype_selector!(
    "Phpolars\\cs\\struct_",
    struct_,
    dtypes(DataTypeSelector::Struct)
);
dtype_selector!("Phpolars\\cs\\string", string, of(&[DataType::String]));
dtype_selector!("Phpolars\\cs\\boolean", boolean, of(&[DataType::Boolean]));
dtype_selector!("Phpolars\\cs\\binary", binary, of(&[DataType::Binary]));
dtype_selector!("Phpolars\\cs\\date", date, of(&[DataType::Date]));
dtype_selector!("Phpolars\\cs\\time", time, of(&[DataType::Time]));
dtype_selector!("Phpolars\\cs\\all", all, sel(Selector::Wildcard));
dtype_selector!("Phpolars\\cs\\empty_", empty, sel(Selector::Empty));
dtype_selector!("Phpolars\\cs\\first", first, sel(polars::prelude::first()));
dtype_selector!("Phpolars\\cs\\last", last, sel(polars::prelude::last()));

/// List columns, optionally only those whose inner type matches a selector.
#[php_function]
#[php(name = "Phpolars\\cs\\list_")]
pub fn list(inner: Option<&PhpExpr>) -> PhpResult<PhpExpr> {
    let inner = match inner {
        None => None,
        Some(e) => Some(std::sync::Arc::new(inner_dtype_selector(e)?)),
    };
    Ok(dtypes(DataTypeSelector::List(inner)))
}

/// Array columns, optionally by inner selector and width.
#[php_function]
#[php(name = "Phpolars\\cs\\array_")]
pub fn array(inner: Option<&PhpExpr>, width: Option<i64>) -> PhpResult<PhpExpr> {
    let inner = match inner {
        None => None,
        Some(e) => Some(std::sync::Arc::new(inner_dtype_selector(e)?)),
    };
    let width = width.map(|w| crate::input::count(w, "width")).transpose()?;
    Ok(dtypes(DataTypeSelector::Array(inner, width)))
}

fn inner_dtype_selector(e: &PhpExpr) -> PhpResult<DataTypeSelector> {
    match &e.inner {
        Expr::Selector(s) => s
            .to_dtype_selector()
            .ok_or_else(|| value_error("the inner selector must select by data type")),
        _ => Err(value_error(
            "the inner selector must be a data type selector",
        )),
    }
}

fn time_units(u: Option<&Zval>) -> PhpResult<TimeUnitSet> {
    match crate::input::given(u) {
        None => Ok(TimeUnitSet::all()),
        Some(v) => {
            let mut set = TimeUnitSet::empty();
            for n in names(&[v])? {
                set |= match crate::input::time_unit(Some(n))? {
                    TimeUnit::Nanoseconds => TimeUnitSet::NANO_SECONDS,
                    TimeUnit::Microseconds => TimeUnitSet::MICRO_SECONDS,
                    TimeUnit::Milliseconds => TimeUnitSet::MILLI_SECONDS,
                };
            }
            Ok(set)
        }
    }
}

/// Datetime columns, optionally by time unit(s) and time zone(s); a null
/// in timeZone matches columns without a zone, '*' any zone.
#[php_function]
#[php(name = "Phpolars\\cs\\datetime")]
pub fn datetime(timeUnit: Option<&Zval>, timeZone: Option<&Zval>) -> PhpResult<PhpExpr> {
    let units = time_units(timeUnit)?;
    let zones = match timeZone {
        None => TimeZoneSet::Any,
        Some(v) if v.is_null() => TimeZoneSet::Unset,
        Some(v) => {
            let mut unset = false;
            let mut zs: Vec<TimeZone> = Vec::new();
            let items: Vec<&Zval> = match v.array() {
                Some(a) => a.values().collect(),
                None => vec![v],
            };
            for z in items {
                match z.str() {
                    None if z.is_null() => unset = true,
                    Some("*") => {
                        return Ok(dtypes(DataTypeSelector::Datetime(
                            units,
                            TimeZoneSet::AnySet,
                        )));
                    }
                    Some(s) => zs.push(
                        TimeZone::opt_try_new(Some(s))
                            .map_err(crate::error::polars_error)?
                            .ok_or_else(|| value_error("empty time zone"))?,
                    ),
                    None => return Err(value_error("timeZone takes strings or null")),
                }
            }
            if unset {
                TimeZoneSet::UnsetOrAnyOf(zs.into())
            } else {
                TimeZoneSet::AnyOf(zs.into())
            }
        }
    };
    Ok(dtypes(DataTypeSelector::Datetime(units, zones)))
}

#[php_function]
#[php(name = "Phpolars\\cs\\duration")]
pub fn duration(timeUnit: Option<&Zval>) -> PhpResult<PhpExpr> {
    Ok(dtypes(DataTypeSelector::Duration(time_units(timeUnit)?)))
}

/// Columns of any of the given types: by_dtype('Int64', 'String').
#[php_function]
#[php(name = "Phpolars\\cs\\by_dtype")]
pub fn by_dtype(dtypes_: &[&Zval]) -> PhpResult<PhpExpr> {
    let mut ds = Vec::new();
    for n in names(dtypes_)? {
        ds.push(crate::dtype::parse(&n)?);
    }
    Ok(of(&ds))
}

/// Columns by name, in the order given: by_name('a') or by_name(['a', 'b']).
/// requireAll: false skips missing names.
#[php_function]
#[php(name = "Phpolars\\cs\\by_name")]
pub fn by_name_fn(names_: &Zval, requireAll: Option<bool>) -> PhpResult<PhpExpr> {
    Ok(sel(by_name(
        names(&[names_])?,
        requireAll.unwrap_or(true),
        false,
    )))
}

/// Columns by position: by_index(0) or by_index([0, -1]); negative
/// positions count from the end.
#[php_function]
#[php(name = "Phpolars\\cs\\by_index")]
pub fn by_index(indices: &Zval, requireAll: Option<bool>) -> PhpResult<PhpExpr> {
    let mut idx: Vec<i64> = Vec::new();
    for v in [indices] {
        match (v.long(), v.array()) {
            (Some(i), _) => idx.push(i),
            (None, Some(a)) => {
                for x in a.values() {
                    idx.push(
                        x.long()
                            .ok_or_else(|| value_error("by_index() takes integers"))?,
                    );
                }
            }
            _ => return Err(value_error("by_index() takes integers")),
        }
    }
    Ok(sel(Selector::ByIndex {
        indices: idx.into(),
        strict: requireAll.unwrap_or(true),
    }))
}

#[php_function]
#[php(name = "Phpolars\\cs\\starts_with")]
pub fn starts_with(prefix: &[&Zval]) -> PhpResult<PhpExpr> {
    Ok(any_of_pattern(names(prefix)?, "^", ".*$"))
}

#[php_function]
#[php(name = "Phpolars\\cs\\ends_with")]
pub fn ends_with(suffix: &[&Zval]) -> PhpResult<PhpExpr> {
    Ok(any_of_pattern(names(suffix)?, "^.*", "$"))
}

/// Columns whose name contains any of the substrings.
#[php_function]
#[php(name = "Phpolars\\cs\\contains")]
pub fn contains(substring: &[&Zval]) -> PhpResult<PhpExpr> {
    Ok(any_of_pattern(names(substring)?, "^.*", ".*$"))
}

/// Columns whose name matches a regex (unanchored, as in py).
#[php_function]
#[php(name = "Phpolars\\cs\\matches")]
pub fn matches(pattern: String) -> PhpExpr {
    let p = match (pattern.starts_with('^'), pattern.ends_with('$')) {
        (true, true) => pattern,
        (true, false) => format!("{pattern}.*$"),
        (false, true) => format!("^.*{pattern}"),
        (false, false) => format!("^.*{pattern}.*$"),
    };
    sel(Selector::Matches(p.into()))
}

/// Every column except the given names or selector.
#[php_function]
#[php(name = "Phpolars\\cs\\exclude")]
pub fn exclude(columns: &[&Zval]) -> PhpResult<PhpExpr> {
    let mut out = Selector::Wildcard;
    for c in columns {
        let s = match as_selector(c, true) {
            Some(s) => s,
            None => by_name(
                names(&[c]).map_err(|_| value_error("exclude() takes names or selectors"))?,
                true,
                false,
            ),
        };
        out -= s;
    }
    Ok(sel(out))
}

pub fn register(module: ModuleBuilder) -> ModuleBuilder {
    module
        .function(wrap_function!(numeric))
        .function(wrap_function!(integer))
        .function(wrap_function!(signed_integer))
        .function(wrap_function!(unsigned_integer))
        .function(wrap_function!(float))
        .function(wrap_function!(decimal))
        .function(wrap_function!(temporal))
        .function(wrap_function!(categorical))
        .function(wrap_function!(enum_))
        .function(wrap_function!(nested))
        .function(wrap_function!(struct_))
        .function(wrap_function!(string))
        .function(wrap_function!(boolean))
        .function(wrap_function!(binary))
        .function(wrap_function!(date))
        .function(wrap_function!(time))
        .function(wrap_function!(all))
        .function(wrap_function!(empty))
        .function(wrap_function!(first))
        .function(wrap_function!(last))
        .function(wrap_function!(list))
        .function(wrap_function!(array))
        .function(wrap_function!(datetime))
        .function(wrap_function!(duration))
        .function(wrap_function!(by_dtype))
        .function(wrap_function!(by_name_fn))
        .function(wrap_function!(by_index))
        .function(wrap_function!(starts_with))
        .function(wrap_function!(ends_with))
        .function(wrap_function!(contains))
        .function(wrap_function!(matches))
        .function(wrap_function!(exclude))
}
