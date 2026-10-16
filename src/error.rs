//! Exceptions mirror py-polars' polars.exceptions names.
use ext_php_rs::{prelude::*, zend::ce};
use polars::prelude::PolarsError;

#[php_class]
#[php(name = "Phpolars\\PolarsError")]
#[php(extends(ce = ce::exception, stub = "\\Exception"))]
#[derive(Default)]
pub struct PolarsErrorBase;

macro_rules! errors {
    ($($rust:ident => $php:literal),* $(,)?) => {
        $(
            #[php_class]
            #[php(name = $php)]
            #[php(extends(PolarsErrorBase))]
            #[derive(Default)]
            pub struct $rust;
        )*

        pub fn register(module: ModuleBuilder) -> ModuleBuilder {
            module.class::<PolarsErrorBase>()$(.class::<$rust>())*
        }
    };
}

errors! {
    ColumnNotFoundError => "Phpolars\\Exception\\ColumnNotFoundError",
    ComputeError => "Phpolars\\Exception\\ComputeError",
    DuplicateError => "Phpolars\\Exception\\DuplicateError",
    InvalidOperationError => "Phpolars\\Exception\\InvalidOperationError",
    NoDataError => "Phpolars\\Exception\\NoDataError",
    OutOfBoundsError => "Phpolars\\Exception\\OutOfBoundsError",
    SchemaError => "Phpolars\\Exception\\SchemaError",
    SchemaFieldNotFoundError => "Phpolars\\Exception\\SchemaFieldNotFoundError",
    ShapeError => "Phpolars\\Exception\\ShapeError",
    StructFieldNotFoundError => "Phpolars\\Exception\\StructFieldNotFoundError",
}

fn root(e: &PolarsError) -> &PolarsError {
    match e {
        PolarsError::Context { error, .. } => root(error),
        e => e,
    }
}

pub fn polars_error(e: PolarsError) -> PhpException {
    let message = e.to_string();
    match root(&e) {
        PolarsError::ColumnNotFound(_) => PhpException::from_class::<ColumnNotFoundError>(message),
        PolarsError::Duplicate(_) => PhpException::from_class::<DuplicateError>(message),
        PolarsError::InvalidOperation(_) => PhpException::from_class::<InvalidOperationError>(message),
        PolarsError::NoData(_) => PhpException::from_class::<NoDataError>(message),
        PolarsError::OutOfBounds(_) => PhpException::from_class::<OutOfBoundsError>(message),
        PolarsError::SchemaMismatch(_) => PhpException::from_class::<SchemaError>(message),
        PolarsError::SchemaFieldNotFound(_) => PhpException::from_class::<SchemaFieldNotFoundError>(message),
        PolarsError::ShapeMismatch(_) => PhpException::from_class::<ShapeError>(message),
        PolarsError::StructFieldNotFound(_) => PhpException::from_class::<StructFieldNotFoundError>(message),
        _ => PhpException::from_class::<ComputeError>(message),
    }
}

/// Bad arguments from PHP: wrong types, unknown options.
pub fn value_error(message: impl Into<String>) -> PhpException {
    PhpException::new(message.into(), 0, ce::value_error())
}

pub fn column_not_found(name: &str) -> PhpException {
    PhpException::from_class::<ColumnNotFoundError>(format!("unable to find column \"{name}\""))
}

pub trait OrPhp<T> {
    fn php(self) -> PhpResult<T>;
}

impl<T> OrPhp<T> for Result<T, PolarsError> {
    fn php(self) -> PhpResult<T> {
        self.map_err(polars_error)
    }
}
