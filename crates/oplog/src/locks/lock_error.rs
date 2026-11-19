//! Every way taking, reading or releasing a lock can be refused.
//! A bare type; lock_error_display_tests exercises each variant's text, lock_error_removed_guards_tests and
//! lock_error_with_removed_guards_tests its takeover guard list.

use std::path::PathBuf;

use super::LockRecord;

/// A lock refusal. Each variant carries what its message needs to name the holder and the file.
/// The variants acquire can return after removing a takeover guard carry `removed_guards`, so every removal is
/// logged even when acquire refuses. BadName, BadRecord, HostUnknown and NotOurs carry none: acquire
/// returns BadName (from lock_path) and BadRecord (encoding `me` on its first write) only before any removal, it
/// never returns HostUnknown, and NotOurs comes from release.
#[derive(Debug)]
pub enum LockError {
    /// The lock file exists and its holder is alive or on another host.
    /// `removed_guards`: the takeover guards this acquire removed before refusing, in order; empty outside acquire.
    Held {
        path: PathBuf,
        holder: LockRecord,
        removed_guards: Vec<LockRecord>,
    },
    /// A guard is in the way: the `<lock>.takeover` guard with a live or other-host holder, or a `.reap-` guard
    /// (another taker removing a dead guard, or left by a crash). `path` is that guard, `holder` its contents.
    /// `removed_guards`: the takeover guards this acquire removed before refusing, in order; empty outside acquire.
    TakeoverInProgress {
        path: PathBuf,
        holder: LockRecord,
        removed_guards: Vec<LockRecord>,
    },
    /// Release of a lock this process does not hold; `holder` is `None` when there is no lock file.
    NotOurs {
        path: PathBuf,
        holder: Option<LockRecord>,
    },
    /// The lock kept changing hands while acquiring it; every attempt found it moved.
    /// `removed_guards`: the takeover guards this acquire removed before refusing, in order; empty outside acquire.
    Contended {
        path: PathBuf,
        removed_guards: Vec<LockRecord>,
    },
    /// A zone name that cannot be part of a lock file name.
    BadName(String),
    /// A record `encode_lock_record` refused to write.
    BadRecord(String),
    /// Lock file text that is not a record; `path` is `None` when decoded without a file.
    /// `removed_guards`: the takeover guards this acquire removed before refusing, in order; empty outside acquire.
    Malformed {
        path: Option<PathBuf>,
        reason: String,
        removed_guards: Vec<LockRecord>,
    },
    /// The host name could not be read.
    HostUnknown(String),
    /// A file system call failed.
    /// `removed_guards`: the takeover guards this acquire removed before refusing, in order; empty outside acquire.
    Io {
        path: PathBuf,
        source: std::io::Error,
        removed_guards: Vec<LockRecord>,
    },
}
