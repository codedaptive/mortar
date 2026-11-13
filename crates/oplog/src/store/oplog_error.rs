//! Every way reading, writing or settling the op log can be refused.
//! A bare type; oplog_error_display_tests exercises each variant's text, oplog_error_source_tests its source.

use std::path::PathBuf;

use gitio::{GitError, Oid};

use super::RefMove;

/// An op log refusal. Debug only: io::Error is neither Clone nor Eq.
#[derive(Debug)]
pub enum OplogError {
    /// Text that is not what mortar writes: a log line (`line` 1-based), a record, or git's answer.
    Malformed { line: Option<usize>, reason: String },
    /// A file system call on `path` failed.
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    /// A git run failed.
    Git(GitError),
    /// An open intent whose refs are neither all at new nor all at old: each move with the ref's actual value.
    Mixed {
        op_id: String,
        key: String,
        refs: Vec<(RefMove, Option<Oid>)>,
    },
    /// An op with a move whose ref name is empty or holds a NUL, found before anything is written: the ref
    /// and why.
    BadMove { refname: String, reason: String },
}
