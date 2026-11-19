//! The clock, in the unit the op log records.
//! No unit test: it reads the system clock. The pure decision and record builders take the time as an argument;
//! the entry points that call them (apply_with_values, with_lock, undo, restore, recover, close_intent) read the
//! clock here.
//! Covered by the crate's integration tests (tests/oplog/recoverable.rs): a record appended by recover
//! carries an `at` between the times read before and after the call. Integration tests should cover: a
//! record appended by apply carries an `at` between the times read before and after the call.

use std::time::{SystemTime, UNIX_EPOCH};

/// Seconds since the unix epoch; 0 for a clock set before it.
pub fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
