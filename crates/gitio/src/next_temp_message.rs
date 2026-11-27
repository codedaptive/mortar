//! next_temp_message: a fresh path for a temporary commit message file under the system temp folder.

use crate::remove_on_drop::RemoveOnDrop;
use crate::temp_message_path::temp_message_path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Counts this process's temporary message files, so two at once never share one.
static SERIAL: AtomicU64 = AtomicU64::new(0);

/// The next temporary message path, named by temp_message_path from this process's id, the time and the serial. It
/// creates no file; the returned RemoveOnDrop removes whatever is written there when it is dropped.
pub(crate) fn next_temp_message() -> RemoveOnDrop {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
    RemoveOnDrop(temp_message_path(
        &std::env::temp_dir(),
        std::process::id(),
        stamp,
        SERIAL.fetch_add(1, Ordering::Relaxed),
    ))
}
