//! Restore's ref name check on bare names, run before any ref is read; public so the cli's restore runs the same check.
//! Its cases that reach git are proved by crates/oplog/tests/oplog/restore_ref_names.rs (git check-ref-format, S-5);
//! check_restore_ref_names_tests covers the names refused before git runs.

use gitio::Git;

use super::name_moves;
use crate::apply::check_ref_names;
use crate::store::OplogError;

/// An empty, NUL-holding or non-refs/ name in `refs` is `BadMove` before git runs; then git check-ref-format is
/// asked about each name in order, and the first it refuses is `BadMove`. A git failure is `OplogError::Git`.
/// Nothing is read or written.
pub fn check_restore_ref_names(git: &Git, refs: &[String]) -> Result<(), OplogError> {
    check_ref_names(git, &name_moves(refs)?)
}
