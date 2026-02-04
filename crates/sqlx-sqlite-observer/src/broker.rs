//! Transaction-aware observation broker for buffering and publishing changes.
//!
//! This module provides transaction-safe change notifications. Changes are buffered
//! during transactions (explicit and implicit) and only published after successful
//! commit. Rolled-back transactions produce no notifications.
//!
//! # Data Flow
//!
//! ```text
//! ┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
//! │ preupdate_hook  │────►│  broker.buffer  │     │   subscribers   │
//! │ (captures data) │     │  (Vec<Event>)   │     │                 │
//! └─────────────────┘     └────────┬────────┘     └─────────────────┘
//!                                  │                       ▲
//!                     ┌────────────┼────────────┐          │
//!                     │            │            │          │
//!               ┌─────▼────┐  ┌────▼─────┐      │          │
//!               │  COMMIT  │  │ ROLLBACK │      │          │
//!               └─────┬────┘  └────┬─────┘      │          │
//!                     │            │            │          │
//!                     ▼            ▼            │          │
//!               on_commit()   on_rollback()     │          │
//!                     │            │            │          │
//!                     │       buffer.clear()    │          │
//!                     │       (discard)         │          │
//!                     │                         │          │
//!                     └─────────────────────────┴──────────┘
//!                             change_tx.send()
//!                             (publish)
//! ```
//!
//! Changes captured by the preupdate hook are buffered until the transaction
//! completes. On commit, buffered changes are published to subscribers. On
//! rollback (or implicit rollback due to an error), they are discarded without
//! notification.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;

use parking_lot::{Mutex, RwLock};
use tokio::sync::broadcast;
use tracing::{debug, trace};

use crate::change::{ChangeOperation, TableChange};
use crate::hooks::{PreUpdateEvent, SqliteValue};

/// Transaction-aware observation broker.
///
/// Buffers preupdate events during transactions and publishes them to
/// subscribers only after successful commit. Rolled-back transactions
/// have their buffered changes discarded.
pub struct ObservationBroker {
   buffer: Mutex<Vec<PreUpdateEvent>>,
   change_tx: broadcast::Sender<TableChange>,
   observed_tables: RwLock<HashSet<String>>,
   capture_values: bool,
}

impl ObservationBroker {
   /// Creates a new broker with the specified broadcast channel capacity.
   pub fn new(channel_capacity: usize, capture_values: bool) -> Arc<Self> {
      let (change_tx, _) = broadcast::channel(channel_capacity);
      Arc::new(Self {
         buffer: Mutex::new(Vec::new()),
         change_tx,
         observed_tables: RwLock::new(HashSet::new()),
         capture_values,
      })
   }

   /// Checks if a table is being observed.
   pub fn is_table_observed(&self, table: &str) -> bool {
      self.observed_tables.read().contains(table)
   }

   /// Registers tables for observation.
   ///
   /// Only changes to observed tables will be buffered and published.
   pub fn observe_tables<I, S>(&self, tables: I)
   where
      I: IntoIterator<Item = S>,
      S: AsRef<str>,
   {
      let mut observed = self.observed_tables.write();
      for table in tables {
         let table_name = table.as_ref().to_string();
         trace!(table = %table_name, "Observing table");
         observed.insert(table_name);
      }
   }

   /// Returns a list of all observed tables.
   pub fn get_observed_tables(&self) -> Vec<String> {
      self.observed_tables.read().iter().cloned().collect()
   }

   /// Called by preupdate_hook - buffers the event for later processing.
   ///
   /// Events are held in the buffer until either `on_commit()` (publish)
   /// or `on_rollback()` (discard) is called.
   pub fn on_preupdate(&self, event: PreUpdateEvent) {
      trace!(
          table = %event.table,
          operation = ?event.operation,
          "Buffering preupdate event"
      );
      self.buffer.lock().push(event);
   }

   /// Called by commit_hook - flushes buffered events to subscribers.
   ///
   /// Converts all buffered `PreUpdateEvent`s to `TableChange`s and sends
   /// them through the broadcast channel. The buffer is cleared afterward.
   pub fn on_commit(&self) {
      let events: Vec<PreUpdateEvent> = {
         let mut buffer = self.buffer.lock();
         std::mem::take(&mut *buffer)
      };

      if events.is_empty() {
         return;
      }

      debug!(count = events.len(), "Flushing buffered changes on commit");

      for event in events {
         let table_change = self.event_to_change(event);
         let _ = self.change_tx.send(table_change);
      }
   }

   /// Called by rollback_hook - discards all buffered events.
   ///
   /// Clears the buffer without publishing any changes to subscribers.
   pub fn on_rollback(&self) {
      let count = {
         let mut buffer = self.buffer.lock();
         let count = buffer.len();
         buffer.clear();
         count
      };

      if count > 0 {
         debug!(count, "Discarding buffered changes on rollback");
      }
   }

   /// Subscribes to change notifications.
   ///
   /// Returns a broadcast receiver that will receive `TableChange` events
   /// after transactions commit.
   pub fn subscribe(&self) -> broadcast::Receiver<TableChange> {
      self.change_tx.subscribe()
   }

   /// Converts a PreUpdateEvent to a TableChange for broadcast.
   fn event_to_change(&self, event: PreUpdateEvent) -> TableChange {
      let rowid = match event.operation {
         ChangeOperation::Insert => Some(event.new_rowid),
         ChangeOperation::Delete => Some(event.old_rowid),
         ChangeOperation::Update => Some(event.new_rowid),
      };

      let (old_values, new_values) = if self.capture_values {
         (
            event.old_values.map(Self::values_to_vec),
            event.new_values.map(Self::values_to_vec),
         )
      } else {
         (None, None)
      };

      TableChange {
         table: event.table,
         operation: Some(event.operation),
         rowid,
         old_values,
         new_values,
         timestamp: Instant::now(),
      }
   }

   /// Converts SqliteValue vec to ColumnValue vec for TableChange.
   fn values_to_vec(values: Vec<SqliteValue>) -> Vec<crate::change::ColumnValue> {
      values.into_iter().map(|v| v.into()).collect()
   }
}

impl std::fmt::Debug for ObservationBroker {
   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      f.debug_struct("ObservationBroker")
         .field("buffer_len", &self.buffer.lock().len())
         .field("observed_tables", &self.observed_tables.read().len())
         .finish()
   }
}
