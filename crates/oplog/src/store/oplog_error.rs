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
    /// An open intent where some refs whose moves change something are at new and some are not: each move with
    /// the ref's actual value.
    Mixed {
        op_id: String,
        key: String,
        refs: Vec<(RefMove, Option<Oid>)>,
    },
    /// An op with a move whose ref name is empty, holds a NUL, is not a full refs/ name, or is refused by git
    /// check-ref-format, found before anything is written: the ref and why.
    BadMove { refname: String, reason: String },
    /// A move of a protected ref that is neither a fast-forward nor a creation: `current` is what the ref
    /// holds (None: absent), `new` what the op asked for (all-zero: a delete), `by` who protects it.
    Protected {
        refname: String,
        current: Option<Oid>,
        new: Oid,
        by: String,
    },
    /// undo found no done op with a move of any of these refs.
    NothingToUndo { refs: Vec<String> },
    /// undo's target op `op_id` left `refname` at `left` (None: absent), but it now holds `actual`; `moved_by` is
    /// the op id and actor of the latest done op after it that moved the ref, or None when no logged op explains
    /// the move.
    MovedSince {
        op_id: String,
        refname: String,
        left: Option<Oid>,
        actual: Option<Oid>,
        moved_by: Option<(String, String)>,
    },
    /// restore named an op id the log does not hold.
    UnknownOp { op_id: String },
    /// restore cannot tell what `refname` held as of `op_id`: no done op in the log moved it.
    ValueUnknown { op_id: String, refname: String },
    /// close_intent named an op_id with no open intent in the log (never written, or already closed);
    /// nothing was appended.
    NotOpen { op_id: String },
}
