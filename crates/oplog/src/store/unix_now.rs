//! The clock, in the unit the op log records.
//! No unit test: it reads the system clock (S-5), and the functions that use a time take it as an argument.
//! Nagatha's integration group should cover: a record appended by apply or recover carries an `at` between
//! the times read before and after the call.

use std::time::{SystemTime, UNIX_EPOCH};

/// Seconds since the unix epoch; 0 for a clock set before it.
pub fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
