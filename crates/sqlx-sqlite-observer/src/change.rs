use std::time::Instant;

use crate::hooks::SqliteValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChangeOperation {
   Insert,
   Update,
   Delete,
}

/// Typed column value from SQLite.
///
/// Represents a single column's value with its native SQLite type.
/// This replaces the previous JSON string representation for better
/// type safety and performance.
#[derive(Debug, Clone, PartialEq)]
pub enum ColumnValue {
   Null,
   Integer(i64),
   Real(f64),
   Text(String),
   Blob(Vec<u8>),
}

impl From<SqliteValue> for ColumnValue {
   fn from(value: SqliteValue) -> Self {
      match value {
         SqliteValue::Null => ColumnValue::Null,
         SqliteValue::Integer(i) => ColumnValue::Integer(i),
         SqliteValue::Real(r) => ColumnValue::Real(r),
         SqliteValue::Text(s) => ColumnValue::Text(s),
         SqliteValue::Blob(b) => ColumnValue::Blob(b),
      }
   }
}

impl ColumnValue {
   /// Returns true if this value is null.
   pub fn is_null(&self) -> bool {
      matches!(self, ColumnValue::Null)
   }

   /// Attempts to get this value as an integer.
   pub fn as_integer(&self) -> Option<i64> {
      match self {
         ColumnValue::Integer(i) => Some(*i),
         _ => None,
      }
   }

   /// Attempts to get this value as a float.
   pub fn as_real(&self) -> Option<f64> {
      match self {
         ColumnValue::Real(r) => Some(*r),
         _ => None,
      }
   }

   /// Attempts to get this value as a string reference.
   pub fn as_text(&self) -> Option<&str> {
      match self {
         ColumnValue::Text(s) => Some(s),
         _ => None,
      }
   }

   /// Attempts to get this value as a blob reference.
   pub fn as_blob(&self) -> Option<&[u8]> {
      match self {
         ColumnValue::Blob(b) => Some(b),
         _ => None,
      }
   }
}

/// Notification of a change to a database table.
///
/// Contains the table name, operation type, affected rowid, and the
/// old/new column values (when available). Changes are only sent after
/// the transaction commits successfully.
#[derive(Debug, Clone)]
pub struct TableChange {
   pub table: String,
   pub operation: Option<ChangeOperation>,
   pub rowid: Option<i64>,
   /// Column values before the change (for UPDATE and DELETE).
   /// Values are ordered by column index as defined in the table schema.
   pub old_values: Option<Vec<ColumnValue>>,
   /// Column values after the change (for INSERT and UPDATE).
   /// Values are ordered by column index as defined in the table schema.
   pub new_values: Option<Vec<ColumnValue>>,
   pub timestamp: Instant,
}
