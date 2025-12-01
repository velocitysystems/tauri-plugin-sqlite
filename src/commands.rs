//! SQLite plugin commands
//!
//! This module implements the Tauri command handlers that the frontend calls.
//! Each command manages database connections through the DbInstances state.

use indexmap::IndexMap;
use serde::Deserialize;
use serde_json::Value as JsonValue;
use sqlx_sqlite_conn_mgr::SqliteDatabaseConfig;
use tauri::{AppHandle, Runtime, State};

use crate::{DbInstances, Error, Result, WriteQueryResult, wrapper::DatabaseWrapper};

/// Statement in a transaction with query and bind values
#[derive(Debug, Deserialize)]
pub struct Statement {
   query: String,
   values: Vec<JsonValue>,
}

/// Load/connect to a database and store it in plugin state.
///
/// If the database is already loaded, returns the existing connection.
/// Otherwise, creates a new connection with optional custom configuration.
#[tauri::command]
pub async fn load<R: Runtime>(
   app: AppHandle<R>,
   db_instances: State<'_, DbInstances>,
   db: String,
   custom_config: Option<SqliteDatabaseConfig>,
) -> Result<String> {
   let instances = db_instances.0.read().await;

   // Return cached if db was already loaded
   if instances.contains_key(&db) {
      return Ok(db);
   }

   drop(instances); // Release read lock before acquiring write lock

   let mut instances = db_instances.0.write().await;

   // Double-check in case another thread loaded it while we waited for write lock
   if instances.contains_key(&db) {
      return Ok(db);
   }

   let wrapper = DatabaseWrapper::connect(&db, &app, custom_config).await?;
   instances.insert(db.clone(), wrapper);

   Ok(db)
}

/// Execute a write query (INSERT, UPDATE, DELETE, etc.)
#[tauri::command]
pub async fn execute(
   db_instances: State<'_, DbInstances>,
   db: String,
   query: String,
   values: Vec<JsonValue>,
) -> Result<(u64, i64)> {
   let instances = db_instances.0.read().await;

   let wrapper = instances
      .get(&db)
      .ok_or_else(|| Error::DatabaseNotLoaded(db.clone()))?;

   let result = wrapper.execute(query, values).await?;

   Ok((result.rows_affected, result.last_insert_id))
}

/// Execute multiple write statements atomically within a transaction
#[tauri::command]
pub async fn execute_transaction(
   db_instances: State<'_, DbInstances>,
   db: String,
   statements: Vec<Statement>,
) -> Result<Vec<WriteQueryResult>> {
   let instances = db_instances.0.read().await;

   let wrapper = instances
      .get(&db)
      .ok_or_else(|| Error::DatabaseNotLoaded(db.clone()))?;

   // Convert Statement structs to tuples for wrapper
   let stmt_tuples: Vec<(String, Vec<JsonValue>)> = statements
      .into_iter()
      .map(|s| (s.query, s.values))
      .collect();

   let results = wrapper.execute_transaction(stmt_tuples).await?;

   Ok(results)
}

/// Execute a SELECT query returning all matching rows
#[tauri::command]
pub async fn fetch_all(
   db_instances: State<'_, DbInstances>,
   db: String,
   query: String,
   values: Vec<JsonValue>,
) -> Result<Vec<IndexMap<String, JsonValue>>> {
   let instances = db_instances.0.read().await;

   let wrapper = instances
      .get(&db)
      .ok_or_else(|| Error::DatabaseNotLoaded(db.clone()))?;

   let rows = wrapper.fetch_all(query, values).await?;

   Ok(rows)
}

/// Execute a SELECT query expecting zero or one result
#[tauri::command]
pub async fn fetch_one(
   db_instances: State<'_, DbInstances>,
   db: String,
   query: String,
   values: Vec<JsonValue>,
) -> Result<Option<IndexMap<String, JsonValue>>> {
   let instances = db_instances.0.read().await;

   let wrapper = instances
      .get(&db)
      .ok_or_else(|| Error::DatabaseNotLoaded(db.clone()))?;

   let row = wrapper.fetch_one(query, values).await?;

   Ok(row)
}

/// Close a specific database connection
///
/// Returns `true` if the database was loaded and successfully closed.
/// Returns `false` if the database was not loaded (nothing to close).
#[tauri::command]
pub async fn close(db_instances: State<'_, DbInstances>, db: String) -> Result<bool> {
   let mut instances = db_instances.0.write().await;

   if let Some(wrapper) = instances.remove(&db) {
      wrapper.close().await?;
      Ok(true)
   } else {
      Ok(false) // Database wasn't loaded
   }
}

/// Close all database connections
#[tauri::command]
pub async fn close_all(db_instances: State<'_, DbInstances>) -> Result<()> {
   let mut instances = db_instances.0.write().await;

   // Collect all wrappers to close
   let wrappers: Vec<DatabaseWrapper> = instances.drain().map(|(_, v)| v).collect();

   // Close each connection
   for wrapper in wrappers {
      wrapper.close().await?;
   }

   Ok(())
}

/// Close database connection and remove all database files
///
/// Returns `true` if the database was loaded and successfully removed.
/// Returns `false` if the database was not loaded (nothing to remove).
#[tauri::command]
pub async fn remove(db_instances: State<'_, DbInstances>, db: String) -> Result<bool> {
   let mut instances = db_instances.0.write().await;

   if let Some(wrapper) = instances.remove(&db) {
      wrapper.remove().await?;
      Ok(true)
   } else {
      Ok(false) // Database wasn't loaded
   }
}
