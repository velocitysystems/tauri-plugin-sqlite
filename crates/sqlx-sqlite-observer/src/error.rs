//! Error types for the sqlx-sqlite-observer crate.

/// Errors that can occur during observation operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
   /// Failed to register SQLite hooks.
   #[error("Hook registration failed: {0}")]
   HookRegistration(String),
}
