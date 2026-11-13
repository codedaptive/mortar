//! Every way taking, reading or releasing a lock can be refused.
//! A bare type; lock_error_display_tests exercises each variant's text.

use std::path::PathBuf;

use super::LockRecord;

/// A lock refusal. Each variant carries what its message needs to name the holder and the file.
#[derive(Debug)]
pub enum LockError {
    /// The lock file exists and its holder is alive or on another host.
    Held { path: PathBuf, holder: LockRecord },
    /// The `<lock>.takeover` guard exists: `path` is the guard, `holder` its contents.
    TakeoverInProgress { path: PathBuf, holder: LockRecord },
    /// Release of a lock this process does not hold; `holder` is `None` when there is no lock file.
    NotOurs {
        path: PathBuf,
        holder: Option<LockRecord>,
    },
    /// The lock kept changing hands while acquiring it; every attempt found it moved.
    Contended { path: PathBuf },
    /// A zone name that cannot be part of a lock file name.
    BadName(String),
    /// A record `encode_lock_record` refused to write.
    BadRecord(String),
    /// Lock file text that is not a record; `path` is `None` when decoded without a file.
    Malformed {
        path: Option<PathBuf>,
        reason: String,
    },
    /// The host name could not be read.
    HostUnknown(String),
    /// A file system call failed.
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
}
