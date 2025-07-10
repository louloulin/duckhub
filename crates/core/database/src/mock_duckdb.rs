//! Mock DuckDB implementation for compilation

use duckhub_common::prelude::*;
use std::fmt::Display;

/// Mock Connection
#[derive(Debug, Clone)]
pub struct Connection {
    _path: String,
}

impl Connection {
    pub fn open<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        Ok(Connection {
            _path: path.as_ref().to_string_lossy().to_string(),
        })
    }

    pub fn open_in_memory() -> Result<Self> {
        Ok(Connection {
            _path: ":memory:".to_string(),
        })
    }

    pub fn execute<P: Display>(&self, _sql: &str, _params: &[P]) -> Result<usize> {
        Ok(1)
    }

    pub fn prepare(&self, _sql: &str) -> Result<Statement> {
        Ok(Statement {})
    }
}

/// Mock Statement
#[derive(Debug, Clone)]
pub struct Statement {}

impl Statement {
    pub fn query_row<T, P: Display, F>(&self, _params: &[P], _f: F) -> Result<T>
    where
        F: FnOnce(&Row) -> Result<T>,
        T: Default,
    {
        Ok(T::default())
    }

    pub fn execute<P: Display>(&self, _params: &[P]) -> Result<usize> {
        Ok(1)
    }
}

/// Mock Row
#[derive(Debug, Clone)]
pub struct Row {}

impl Row {
    pub fn get<T>(&self, _idx: usize) -> Result<T>
    where
        T: Default,
    {
        Ok(T::default())
    }

    pub fn get_ref(&self, _idx: usize) -> Result<ValueRef> {
        Ok(ValueRef::Null)
    }
}

/// Mock ValueRef
#[derive(Debug, Clone)]
pub enum ValueRef {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}

/// Mock Error
#[derive(Debug)]
pub enum Error {
    InvalidColumnType(usize, String),
    ExecuteFailed(String),
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidColumnType(idx, msg) => write!(f, "Invalid column type at index {}: {}", idx, msg),
            Error::ExecuteFailed(msg) => write!(f, "Execute failed: {}", msg),
            Error::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

/// Mock DuckDBConnection
#[derive(Debug)]
pub struct DuckDBConnection {
    pub connection: Connection,
}

impl DuckDBConnection {
    pub fn new(path: &str) -> Result<Self> {
        Ok(DuckDBConnection {
            connection: Connection::open(path)?,
        })
    }
}

/// Mock types module
pub mod types {
    pub use super::ValueRef;
}
