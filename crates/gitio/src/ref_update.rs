//! RefUpdate: one ref move in an all-or-nothing update. A declaration only; update_refs_stdin_tests and
//! classify_update_refs_tests exercise it.

use crate::Oid;

/// One ref to move, and the value it must hold before the move.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefUpdate {
    /// The full ref name the caller chose, such as `refs/heads/main`.
    pub refname: String,
    /// The value to set. An all-zero value deletes the ref, which is git's meaning; gitio does not refuse it.
    pub new: Oid,
    /// The value the ref must hold now; None means the ref must not exist.
    pub old: Option<Oid>,
}
