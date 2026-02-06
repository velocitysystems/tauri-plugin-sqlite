use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::sync::broadcast;
use tokio_stream::Stream;
use tokio_stream::wrappers::BroadcastStream;
use tracing::warn;

use crate::change::TableChange;

/// A filtered stream of table change notifications.
///
/// Wraps a `BroadcastStream` with optional table filtering. Uses proper async
/// wakeups instead of busy-polling.
pub struct TableChangeStream {
   inner: BroadcastStream<TableChange>,
   filter_tables: Option<Vec<String>>,
}

impl TableChangeStream {
   pub fn new(rx: broadcast::Receiver<TableChange>) -> Self {
      Self {
         inner: BroadcastStream::new(rx),
         filter_tables: None,
      }
   }

   pub fn filter_tables(mut self, tables: Vec<String>) -> Self {
      self.filter_tables = Some(tables);
      self
   }
}

impl Stream for TableChangeStream {
   type Item = TableChange;

   fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
      loop {
         // BroadcastStream is Unpin, so we can safely create a pinned reference
         let inner = Pin::new(&mut self.inner);

         match inner.poll_next(cx) {
            Poll::Ready(Some(Ok(change))) => {
               if let Some(ref tables) = self.filter_tables
                  && !tables.contains(&change.table)
               {
                  continue;
               }
               return Poll::Ready(Some(change));
            }
            Poll::Ready(Some(Err(err))) => {
               // Lagged error - missed some messages due to slow consumption
               warn!(
                  error = %err,
                  "Stream lagged — missed change notifications. Consider increasing channel_capacity."
               );
               continue;
            }
            Poll::Ready(None) => return Poll::Ready(None),
            Poll::Pending => return Poll::Pending,
         }
      }
   }
}

/// Extension trait for converting broadcast receivers into table change streams.
///
/// Provides a convenient way to convert a `broadcast::Receiver<TableChange>` into
/// a `TableChangeStream` that implements `futures::Stream`.
pub trait TableChangeStreamExt {
   /// Converts this receiver into a `TableChangeStream`.
   ///
   /// The returned stream can be further filtered using [`TableChangeStream::filter_tables`].
   fn into_stream(self) -> TableChangeStream;
}

impl TableChangeStreamExt for broadcast::Receiver<TableChange> {
   fn into_stream(self) -> TableChangeStream {
      TableChangeStream::new(self)
   }
}
