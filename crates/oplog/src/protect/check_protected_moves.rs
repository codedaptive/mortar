//! Checks every move of a protected ref against its current value and merge base.
//! No unit test: it runs git (S-5); protected_move_verdict_tests covers the decision. Nagatha's integration
//! group should cover, in a throwaway repository: a fast-forward of a protected ref passes; a creation of an
//! absent protected ref passes; a rewind, a sideways move, a move to an unrelated commit and a delete of a
//! protected ref return Protected; a move of an unprotected ref is not read and passes whatever it does; a ref
//! listed twice in `protected` is judged by its first entry; a git failure reading a ref or its merge base is
//! OplogError::Git.

use gitio::Git;

use super::{protected_move_verdict, ProtectedRef};
use crate::store::{oid_is_zero, read_ref_value, OplogError, RefMove};

/// The first refusal among `moves`, in order, or Ok when every move of a protected ref is a fast-forward, a
/// creation or no move. Only moves whose ref is in `protected` are read; the merge base is asked only for a
/// present ref moved to another, non-zero value.
///
/// apply_protected runs check_ref_moves first, so names here are full refs/ names; git check-ref-format runs
/// later, in apply. Nothing is written here.
pub(crate) fn check_protected_moves(
    git: &Git,
    moves: &[RefMove],
    protected: &[ProtectedRef],
) -> Result<(), OplogError> {
    for m in moves {
        let Some(guard) = protected.iter().find(|p| p.refname == m.refname) else {
            continue;
        };
        let current = read_ref_value(git, &m.refname)?;
        let merge_base = match &current {
            Some(cur) if !oid_is_zero(&m.new) && m.new != *cur => {
                git.merge_base(cur, &m.new).map_err(OplogError::Git)?
            }
            _ => None,
        };
        protected_move_verdict(m, guard, current.as_ref(), merge_base.as_ref())?;
    }
    Ok(())
}
