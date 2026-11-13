//! A fresh op id for a caller building an Op.
//! No unit test: it reads the clock and the process id (S-5); op_id_from_parts_tests covers the format.
//! Nagatha's integration group should cover: two calls in one process return different ids; ids from two
//! processes started at once differ; each id has the form `<hex>-<hex>-<hex>` with the middle part the
//! process id in hex.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use super::op_id_from_parts;

/// Counts the ids made in this process, so two calls within one clock tick still differ.
static NEXT: AtomicU64 = AtomicU64::new(0);

/// The clock in nanoseconds (0 for a clock set before the epoch), this process's id and the next count.
pub fn new_op_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    op_id_from_parts(nanos, std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed))
}
