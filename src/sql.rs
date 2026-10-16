#![allow(non_snake_case)]
//! SQL over frames, as py-polars' SQLContext, DataFrame.sql(), and pl.sql().
//! Built only with the `sql` Cargo feature; without it these raise a ComputeError.
use ext_php_rs::{prelude::*, types::ZendHashTable, types::Zval};
#[allow(unused_imports)]
use polars::prelude::DataType;
use polars::prelude::*;

use crate::{
    error::OrPhp,
    frame::{PhpDataFrame, PhpLazyFrame},
};

#[cfg(not(feature = "sql"))]
fn missing() -> PhpException {
    PhpException::from_class::<crate::error::ComputeError>(
        "php_polars was built without the 'sql' feature".to_string(),
    )
}

/// A DataFrame or LazyFrame argument as a LazyFrame.
fn lazy(v: &Zval) -> PhpResult<LazyFrame> {
    if let Some(df) = v.extract::<&PhpDataFrame>() {
        return Ok(df.inner.clone().lazy());
    }
    if let Some(lf) = v.extract::<&PhpLazyFrame>() {
        return Ok(lf.inner.clone());
    }
    Err(crate::error::value_error(
        "expected a Phpolars\\DataFrame or Phpolars\\LazyFrame",
    ))
}

/// The query result: a LazyFrame, or a collected DataFrame when eager.
fn result(lf: LazyFrame, eager: bool) -> PhpResult<Zval> {
    use ext_php_rs::convert::IntoZval;
    if eager {
        crate::init_threads();
        Ok(PhpDataFrame::wrap(lf.collect().php()?).into_zval(false)?)
    } else {
        Ok(PhpLazyFrame::wrap(lf).into_zval(false)?)
    }
}

/// Run one query over named frames.
pub fn run(query: &str, frames: &[(String, LazyFrame)]) -> PhpResult<LazyFrame> {
    #[cfg(feature = "sql")]
    {
        let mut ctx = polars::sql::SQLContext::new();
        for (name, lf) in frames {
            ctx.register(name, lf.clone());
        }
        ctx.execute(query).php()
    }
    #[cfg(not(feature = "sql"))]
    {
        let _ = (query, frames);
        Err(missing())
    }
}

fn named_frames(frames: Option<&ZendHashTable>) -> PhpResult<Vec<(String, LazyFrame)>> {
    let mut out = Vec::new();
    if let Some(f) = frames {
        for (k, v) in f.iter() {
            out.push((k.to_string(), lazy(v)?));
        }
    }
    Ok(out)
}

/// new SQLContext(['sales' => $df]); $ctx->execute('SELECT ...').
/// execute() returns a LazyFrame unless eager is set, as in py.
#[php_class]
#[php(name = "Phpolars\\SQLContext")]
pub struct PhpSqlContext {
    frames: Vec<(String, LazyFrame)>,
    eager: bool,
}

#[php_impl]
impl PhpSqlContext {
    pub fn __construct(frames: Option<&ZendHashTable>, eager: Option<bool>) -> PhpResult<Self> {
        Ok(Self {
            frames: named_frames(frames)?,
            eager: eager.unwrap_or(false),
        })
    }

    /// Register a frame under a table name, replacing any with that name.
    pub fn register(&mut self, name: String, frame: &Zval) -> PhpResult<()> {
        let lf = lazy(frame)?;
        self.frames.retain(|(n, _)| n != &name);
        self.frames.push((name, lf));
        Ok(())
    }

    /// Register several frames: registerMany(['a' => $df, 'b' => $lf]).
    pub fn registerMany(&mut self, frames: &ZendHashTable) -> PhpResult<()> {
        for (name, lf) in named_frames(Some(frames))? {
            self.frames.retain(|(n, _)| n != &name);
            self.frames.push((name, lf));
        }
        Ok(())
    }

    pub fn unregister(&mut self, names: &Zval) -> PhpResult<()> {
        let names = crate::input::names(&[names])?;
        self.frames.retain(|(n, _)| !names.contains(n));
        Ok(())
    }

    /// The registered table names, sorted.
    pub fn tables(&self) -> Vec<String> {
        let mut t: Vec<String> = self.frames.iter().map(|(n, _)| n.clone()).collect();
        t.sort();
        t
    }

    pub fn execute(&self, query: String, eager: Option<bool>) -> PhpResult<Zval> {
        result(run(&query, &self.frames)?, eager.unwrap_or(self.eager))
    }
}

/// sql('SELECT * FROM t', ['t' => $df]): one query over named frames.
/// Returns a LazyFrame unless eager: true.
#[php_function]
#[php(name = "Phpolars\\sql")]
pub fn sql(query: String, frames: Option<&ZendHashTable>, eager: Option<bool>) -> PhpResult<Zval> {
    result(run(&query, &named_frames(frames)?)?, eager.unwrap_or(false))
}

pub fn register(module: ModuleBuilder) -> ModuleBuilder {
    module
        .class::<PhpSqlContext>()
        .function(wrap_function!(sql))
}

// Writing to a database through PDO.

fn call(
    obj: &Zval,
    method: &str,
    args: Vec<&dyn ext_php_rs::convert::IntoZvalDyn>,
) -> PhpResult<Zval> {
    Ok(obj.try_call_method(method, args)?)
}

fn sql_type(dt: &DataType, driver: &str) -> &'static str {
    match dt {
        DataType::Boolean => "BOOLEAN",
        d if d.is_integer() => "BIGINT",
        d if d.is_float() || d.is_decimal() => "DOUBLE PRECISION",
        DataType::Date => "DATE",
        // MySQL's TIMESTAMP drops fractions of a second and ends in 2038.
        DataType::Datetime(..) if driver == "mysql" => "DATETIME(6)",
        DataType::Datetime(..) => "TIMESTAMP",
        _ => "TEXT",
    }
}

/// A cell as a PDO parameter: dates as ISO text, booleans as 1/0, nested values as JSON.
fn param(v: &Zval, dt: &DataType) -> PhpResult<Zval> {
    use ext_php_rs::convert::IntoZval;
    if v.is_null() {
        return Ok(Zval::new());
    }
    if let Some(b) = v.bool() {
        return Ok((b as i64).into_zval(false)?);
    }
    if v.is_object() && matches!(dt, DataType::Date | DataType::Datetime(..)) {
        let fmt = if matches!(dt, DataType::Date) {
            "Y-m-d"
        } else {
            "Y-m-d H:i:s.u"
        };
        return call(v, "format", vec![&fmt]);
    }
    if v.is_array() {
        let json = ext_php_rs::types::ZendCallable::try_from_name("json_encode")?;
        return Ok(json.try_call(vec![v])?);
    }
    Ok(v.shallow_clone())
}

pub fn write_database(df: &DataFrame, table: &str, pdo: &Zval, if_exists: &str) -> PhpResult<i64> {
    use ext_php_rs::convert::IntoZval;
    // PDO::ATTR_DRIVER_NAME
    let driver = call(pdo, "getAttribute", vec![&16i64])?
        .str()
        .unwrap_or("")
        .to_string();
    let quote = |name: &str| -> String {
        if driver == "mysql" {
            format!("`{}`", name.replace('`', "``"))
        } else {
            format!("\"{}\"", name.replace('"', "\"\""))
        }
    };
    let exists_sql = match driver.as_str() {
        "sqlite" => "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?",
        "pgsql" => {
            "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = ? AND table_schema = current_schema()"
        }
        "mysql" => {
            "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = ? AND table_schema = DATABASE()"
        }
        _ => "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = ?",
    };
    let stmt = call(pdo, "prepare", vec![&exists_sql])?;
    let mut args = ZendHashTable::new();
    args.push(table)?;
    call(&stmt, "execute", vec![&args.into_zval(false)?])?;
    let count = call(&stmt, "fetchColumn", vec![])?;
    // An open cursor keeps SQLite's table locked against DROP TABLE.
    call(&stmt, "closeCursor", vec![])?;
    // Drivers return the count as an int or as a numeric string.
    let exists = count
        .long()
        .or_else(|| count.str().and_then(|s| s.trim().parse().ok()))
        .unwrap_or(0)
        > 0;
    let create = match (exists, if_exists) {
        (true, "fail") => {
            return Err(crate::error::value_error(format!(
                "table '{table}' already exists; pass ifTableExists: 'append' or 'replace'"
            )));
        }
        (true, "append") => false,
        (true, "replace") => {
            call(pdo, "exec", vec![&format!("DROP TABLE {}", quote(table))])?;
            true
        }
        (false, "fail" | "append" | "replace") => true,
        (_, other) => {
            return Err(crate::error::value_error(format!(
                "ifTableExists must be fail, replace, or append, not '{other}'"
            )));
        }
    };
    if create {
        let cols: Vec<String> = df
            .columns()
            .iter()
            .map(|c| format!("{} {}", quote(c.name()), sql_type(c.dtype(), &driver)))
            .collect();
        call(
            pdo,
            "exec",
            vec![&format!(
                "CREATE TABLE {} ({})",
                quote(table),
                cols.join(", ")
            )],
        )?;
    }
    if df.height() == 0 {
        return Ok(0);
    }
    let names: Vec<String> = df.columns().iter().map(|c| quote(c.name())).collect();
    let dtypes: Vec<DataType> = df.columns().iter().map(|c| c.dtype().clone()).collect();
    let width = df.width().max(1);
    // Stay under SQLite's 999-parameter limit.
    let per_statement = (900 / width).max(1);
    let rows = crate::convert::to_tuples(df)?;
    let rows: Vec<&Zval> = rows.values().collect();
    let in_transaction = call(pdo, "beginTransaction", vec![]).is_ok();
    let result = (|| -> PhpResult<i64> {
        for chunk in rows.chunks(per_statement) {
            let placeholders =
                vec![format!("({})", vec!["?"; width].join(", ")); chunk.len()].join(", ");
            let sql = format!(
                "INSERT INTO {} ({}) VALUES {}",
                quote(table),
                names.join(", "),
                placeholders
            );
            let stmt = call(pdo, "prepare", vec![&sql])?;
            let mut params = ZendHashTable::with_capacity((chunk.len() * width) as u32);
            for row in chunk {
                for (v, dt) in row.array().unwrap().values().zip(&dtypes) {
                    params.push(param(v, dt)?)?;
                }
            }
            call(&stmt, "execute", vec![&params.into_zval(false)?])?;
        }
        Ok(rows.len() as i64)
    })();
    if in_transaction {
        match &result {
            Ok(_) => {
                call(pdo, "commit", vec![])?;
            }
            Err(_) => {
                let _ = call(pdo, "rollBack", vec![]);
            }
        }
    }
    result
}
