# SQLx SQLite Observer

Reactive change notifications for SQLite databases using sqlx.

This crate provides **transaction-safe** change notifications for SQLite databases
using SQLite's native hooks (`preupdate_hook`, `commit_hook`, `rollback_hook`).

## Features

   * **Transaction-safe notifications**: Changes only notify after successful commit
   * **Typed column values**: Access old/new values with native SQLite types
   * **Stream support**: Use `tokio_stream::Stream` for async iteration
   * **Multiple subscribers**: Broadcast channel supports multiple listeners
   * **Optional SQLx SQLite Connection Manager integration**: Works with
     `sqlx-sqlite-conn-mgr` for single-writer/multi-reader patterns

## SQLite Requirements

Requires SQLite compiled with `SQLITE_ENABLE_PREUPDATE_HOOK`.

**Important:** Most system SQLite libraries do NOT have this option enabled by
default. You have two options:

1. **Use the `bundled` feature** (recommended for most users):

   ```toml
   sqlx-sqlite-observer = { version = "0.8", features = ["bundled"] }
   ```

   This compiles SQLite from source with preupdate hook support (~1MB binary size
   increase).

2. **Provide your own SQLite** with `SQLITE_ENABLE_PREUPDATE_HOOK` compiled in.
   Use `is_preupdate_hook_enabled()` to verify at runtime.

If preupdate hooks are not available, `SqliteObserver::acquire()` will return an
error with a descriptive message.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
sqlx-sqlite-observer = "0.8"
```

For integration with `sqlx-sqlite-conn-mgr`:

```toml
[dependencies]
sqlx-sqlite-observer = { version = "0.8", features = ["conn-mgr"] }
```

## How It Works

The library uses SQLite's native hooks for transaction-safe change tracking:

```text
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│ preupdate_hook  │────►│  broker.buffer  │     │   subscribers   │
│ (captures data) │     │  (Vec<Event>)   │     │                 │
└─────────────────┘     └────────┬────────┘     └─────────────────┘
                                 │                       ▲
                    ┌────────────┼────────────┐          │
                    │            │            │          │
              ┌─────▼────┐  ┌────▼─────┐      │          │
              │  COMMIT  │  │ ROLLBACK │      │          │
              └─────┬────┘  └────┬─────┘      │          │
                    │            │            │          │
                    ▼            ▼            │          │
              on_commit()   on_rollback()     │          │
                    │            │            │          │
                    │       buffer.clear()    │          │
                    │       (discard)         │          │
                    │                         │          │
                    └─────────────────────────┴──────────┘
                            change_tx.send()
                            (publish)
```

1. When you acquire a connection, observation hooks are registered on the raw
   SQLite handle
2. `preupdate_hook` captures changes (table, operation, old/new values) and
   buffers them
3. `commit_hook` fires when a transaction commits, publishing buffered changes
   to subscribers
4. `rollback_hook` fires when a transaction rolls back, discarding buffered
   changes

This ensures subscribers **only receive notifications for committed changes**.

## API Reference

### Core Types

   * **`TableChange`**: Notification of a change to a database table
   * **`ChangeOperation`**: Insert, Update, or Delete
   * **`ColumnValue`**: Typed column value (Null, Integer, Real, Text, Blob)
   * **`ObserverConfig`**: Configuration for table filtering and channel
     capacity <!-- COMING SOON -->

### Observer Types <!-- COMING SOON -->

   * **`SqliteObserver`**: Main observer for `SqlitePool` connections
   * **`ObservableConnection`**: Connection wrapper with hooks registered

### Stream Types <!-- COMING SOON -->

   * **`TableChangeStream`**: Async stream of table changes
   * **`TableChangeStreamExt`**: Extension trait for converting receivers to
     streams

### SQLx SQLite Connection Manager Integration (feature: `conn-mgr`) <!-- COMING SOON -->

   * **`ObservableSqliteDatabase`**: Wrapper for `SqliteDatabase` with observation
   * **`ObservableWriteGuard`**: Write guard with hooks registered

## Examples

> **Coming in Phase 2** - Full working examples will be added in a subsequent PR.

### Basic Usage

<!-- TODO: Add basic example showing SqliteObserver usage -->

### Stream API

<!-- TODO: Add stream example showing TableChangeStream usage -->

### Value Capture

<!-- TODO: Add example showing old/new column value access -->

### SQLx SQLite Connection Manager Integration

<!-- TODO: Add example showing ObservableSqliteDatabase usage -->

## Usage Notes

### Channel Capacity

The `channel_capacity` in `ObserverConfig` determines how many changes can be
buffered. All changes in a transaction are delivered at once on commit. If your
transaction contains more mutating statements than this capacity, **messages
will be dropped**.

<!-- COMING SOON -->

```rust
let config = ObserverConfig::new()
    .with_tables(["users", "posts"])
    .with_channel_capacity(1000); // Handle large transactions
```

### Disabling Value Capture

By default, `TableChange` includes `old_values` and `new_values` with the actual
column data. Disable this for lower memory usage if you only need row IDs:

<!-- COMING SOON -->

```rust
let config = ObserverConfig::new()
    .with_tables(["users"])
    .with_capture_values(false); // Only track table + rowid
```

## License

MIT License - see [LICENSE](LICENSE) for details.
