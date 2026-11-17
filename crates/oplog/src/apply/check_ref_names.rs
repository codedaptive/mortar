//! Checks every move's ref name: the pure checks, then git check-ref-format.
//! No unit test: it runs git (S-5); check_ref_moves_tests and ref_format_verdict_tests cover its decisions.
//! Nagatha's integration group, in a throwaway repository, should cover: a full well-formed name passes;
//! refs/heads/a..b is BadMove with no intent written; heads/main is BadMove (not refs/).

use gitio::Git;

use super::{check_ref_moves, ref_format_verdict};
use crate::store::{OplogError, RefMove};

/// Runs check_ref_moves first, so git is never asked about an empty name, a name holding a NUL or one outside
/// refs/; then asks git check-ref-format about each move's ref name in order. The first refused name is
/// `BadMove`. A GitError here is a git failure, `OplogError::Git`. Nothing is written either way: apply calls
/// this before the intent.
pub(crate) fn check_ref_names(git: &Git, moves: &[RefMove]) -> Result<(), OplogError> {
    check_ref_moves(moves)?;
    for m in moves {
        ref_format_verdict(
            &m.refname,
            git.check_ref_format(&m.refname).map_err(OplogError::Git)?,
        )?;
    }
    Ok(())
}
