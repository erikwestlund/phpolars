#![allow(non_snake_case)]
//! Phpolars\Config: display settings, as py-polars' pl.Config. They apply to
//! the whole process; under PHP-FPM a worker keeps them across requests.
use ext_php_rs::prelude::*;
use polars_core::fmt as pfmt;

use crate::error::value_error;

const ENV: &[&str] = &[
    "POLARS_FMT_MAX_ROWS",
    "POLARS_FMT_MAX_COLS",
    "POLARS_FMT_STR_LEN",
    "POLARS_FMT_TABLE_CELL_LIST_LEN",
    "POLARS_FMT_TABLE_FORMATTING",
    "POLARS_FMT_TABLE_ROUNDED_CORNERS",
    "POLARS_FMT_TABLE_HIDE_COLUMN_DATA_TYPES",
    "POLARS_FMT_TABLE_HIDE_COLUMN_NAMES",
    "POLARS_FMT_TABLE_HIDE_COLUMN_SEPARATOR",
    "POLARS_FMT_TABLE_HIDE_DATAFRAME_SHAPE_INFORMATION",
    "POLARS_FMT_TABLE_INLINE_COLUMN_DATA_TYPE",
    "POLARS_FMT_TABLE_DATAFRAME_SHAPE_BELOW",
    "POLARS_FMT_TABLE_CELL_ALIGNMENT",
    "POLARS_FMT_TABLE_CELL_NUMERIC_ALIGNMENT",
    "POLARS_TABLE_WIDTH",
];

/// Polars reads these settings from the environment when it prints a table.
fn set(name: &str, value: Option<String>) {
    // SAFETY: PHP NTS runs one request at a time on one thread, and Polars'
    // worker threads are idle between the calls that PHP code makes, so no
    // other thread reads the environment while it changes.
    unsafe {
        match value {
            Some(v) => std::env::set_var(name, v),
            None => std::env::remove_var(name),
        }
    }
}

fn flag(name: &str, on: Option<bool>) {
    set(name, on.map(|b| if b { "1" } else { "0" }.to_string()));
}

fn alignment(a: Option<String>) -> PhpResult<Option<String>> {
    match a.as_deref() {
        None => Ok(None),
        Some(x @ ("LEFT" | "CENTER" | "RIGHT")) => Ok(Some(x.to_string())),
        Some(x) => Err(value_error(format!(
            "alignment must be LEFT, CENTER, or RIGHT, not '{x}'"
        ))),
    }
}

#[php_class]
#[php(name = "Phpolars\\Config")]
pub struct PhpConfig;

#[php_impl]
impl PhpConfig {
    /// Rows shown when printing a frame; null restores the default, -1 shows all.
    pub fn setTblRows(n: Option<i64>) {
        set("POLARS_FMT_MAX_ROWS", n.map(|n| n.to_string()));
    }

    pub fn setTblCols(n: Option<i64>) {
        set("POLARS_FMT_MAX_COLS", n.map(|n| n.to_string()));
    }

    /// The longest string shown in a cell before it is cut.
    pub fn setFmtStrLengths(n: Option<i64>) {
        set("POLARS_FMT_STR_LEN", n.map(|n| n.to_string()));
    }

    pub fn setFmtTableCellListLen(n: Option<i64>) {
        set("POLARS_FMT_TABLE_CELL_LIST_LEN", n.map(|n| n.to_string()));
    }

    /// The table width in characters.
    pub fn setTblWidthChars(n: Option<i64>) {
        set("POLARS_TABLE_WIDTH", n.map(|n| n.to_string()));
    }

    /// A comfy-table preset such as 'ASCII_FULL', 'ASCII_MARKDOWN', or 'NOTHING'.
    pub fn setTblFormatting(format: Option<String>, roundedCorners: Option<bool>) {
        set("POLARS_FMT_TABLE_FORMATTING", format);
        flag("POLARS_FMT_TABLE_ROUNDED_CORNERS", roundedCorners);
    }

    pub fn setTblHideColumnDataTypes(active: Option<bool>) {
        flag("POLARS_FMT_TABLE_HIDE_COLUMN_DATA_TYPES", active);
    }

    pub fn setTblHideColumnNames(active: Option<bool>) {
        flag("POLARS_FMT_TABLE_HIDE_COLUMN_NAMES", active);
    }

    pub fn setTblHideDtypeSeparator(active: Option<bool>) {
        flag("POLARS_FMT_TABLE_HIDE_COLUMN_SEPARATOR", active);
    }

    pub fn setTblHideDataframeShape(active: Option<bool>) {
        flag("POLARS_FMT_TABLE_HIDE_DATAFRAME_SHAPE_INFORMATION", active);
    }

    pub fn setTblColumnDataTypeInline(active: Option<bool>) {
        flag("POLARS_FMT_TABLE_INLINE_COLUMN_DATA_TYPE", active);
    }

    pub fn setTblDataframeShapeBelow(active: Option<bool>) {
        flag("POLARS_FMT_TABLE_DATAFRAME_SHAPE_BELOW", active);
    }

    /// LEFT, CENTER, or RIGHT.
    pub fn setTblCellAlignment(format: Option<String>) -> PhpResult<()> {
        set("POLARS_FMT_TABLE_CELL_ALIGNMENT", alignment(format)?);
        Ok(())
    }

    pub fn setTblCellNumericAlignment(format: Option<String>) -> PhpResult<()> {
        set(
            "POLARS_FMT_TABLE_CELL_NUMERIC_ALIGNMENT",
            alignment(format)?,
        );
        Ok(())
    }

    /// Digits after the decimal point for floats; null restores the default.
    pub fn setFloatPrecision(precision: Option<i64>) -> PhpResult<()> {
        pfmt::set_float_precision(
            precision
                .map(|p| crate::input::count(p, "precision"))
                .transpose()?,
        );
        Ok(())
    }

    /// 'mixed' (the default) or 'full'.
    pub fn setFmtFloat(fmt: Option<String>) -> PhpResult<()> {
        pfmt::set_float_fmt(match fmt.as_deref().unwrap_or("mixed") {
            "mixed" => pfmt::FloatFmt::Mixed,
            "full" => pfmt::FloatFmt::Full,
            f => return Err(value_error(format!("fmt must be mixed or full, not '{f}'"))),
        });
        Ok(())
    }

    /// One character between thousands, such as ','; null turns it off.
    pub fn setThousandsSeparator(separator: Option<String>) -> PhpResult<()> {
        pfmt::set_thousands_separator(one_char(separator, "separator")?);
        Ok(())
    }

    pub fn setDecimalSeparator(separator: Option<String>) -> PhpResult<()> {
        pfmt::set_decimal_separator(one_char(separator, "separator")?);
        Ok(())
    }

    pub fn setTrimDecimalZeros(active: Option<bool>) {
        pfmt::set_trim_decimal_zeros(active);
    }

    /// Undo every Config setting.
    pub fn restoreDefaults() {
        for name in ENV {
            set(name, None);
        }
        pfmt::set_float_precision(None);
        pfmt::set_float_fmt(pfmt::FloatFmt::Mixed);
        pfmt::set_thousands_separator(None);
        pfmt::set_decimal_separator(None);
        pfmt::set_trim_decimal_zeros(None);
    }
}

fn one_char(s: Option<String>, what: &str) -> PhpResult<Option<char>> {
    match s {
        None => Ok(None),
        Some(s) => {
            let mut chars = s.chars();
            match (chars.next(), chars.next()) {
                (Some(c), None) => Ok(Some(c)),
                _ => Err(value_error(format!("{what} must be one character"))),
            }
        }
    }
}
