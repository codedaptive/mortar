//! temp_message_path: where the temporary commit message file of Git::commit_hooks lives.

use std::path::{Path, PathBuf};

/// `<dir>/mortar-message-<pid>-<stamp>-<serial>`. The process id keeps processes apart, the serial keeps two message
/// files of one process apart, and the stamp keeps a reused process id from meeting a file a dead process left behind.
pub(crate) fn temp_message_path(dir: &Path, pid: u32, stamp: u128, serial: u64) -> PathBuf {
    dir.join(format!("mortar-message-{pid}-{stamp}-{serial}"))
}
