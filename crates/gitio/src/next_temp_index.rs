//! next_temp_index: a fresh path for a temporary index under the system temp folder.

use crate::remove_on_drop::RemoveOnDrop;
use crate::temp_index_path::temp_index_path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Counts this process's temporary indexes, so two at once never share one.
static SERIAL: AtomicU64 = AtomicU64::new(0);

/// The next temporary index path, named by temp_index_path from this process's id, the time and the serial. It
/// creates no file; the returned RemoveOnDrop removes whatever is written there when it is dropped.
pub(crate) fn next_temp_index() -> RemoveOnDrop {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
    RemoveOnDrop(temp_index_path(
        &std::env::temp_dir(),
        std::process::id(),
        stamp,
        SERIAL.fetch_add(1, Ordering::Relaxed),
    ))
}
