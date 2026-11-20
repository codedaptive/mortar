//! BackupError: why a backup or restore_backup was refused. A declaration only;
//! backup_error_display_tests exercises the text of every variant,
//! backup_error_error_tests its source, backup_prefix_tests BadHost.

use gitio::{GitError, PushRefusal};
use oplog::{LockError, OpRecord, OplogError};

/// A backup or restore refusal. Debug only: LockError and OplogError are neither Clone nor Eq.
#[derive(Debug)]
pub enum BackupError {
    /// This machine's host name, which names the backup namespace, could not be read.
    Host(LockError),
    /// The host name is empty or holds '/', so it cannot name a namespace of its own; no git was run.
    BadHost(String),
    /// A git run failed.
    Git(GitError),
    /// The remote refused to move one backup ref; `refname` is that backup ref's name on the remote.
    Refused { refname: String, refusal: PushRefusal },
    /// Reading or writing the op log failed, or git failed inside apply.
    Oplog(OplogError),
    /// A local ref was not at its expected value, so the restore op moved nothing; its `failed` record.
    OpFailed(OpRecord),
    /// The log already held a done op with this attempt's key, so this restore moved nothing; that done record.
    OpRepeated(OpRecord),
}
