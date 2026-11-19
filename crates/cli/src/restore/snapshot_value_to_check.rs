//! snapshot_value_to_check: which snapshot value restore's saved check compares a folder against, the done record's
//! or the snapshot ref's present one.
//! ZUT snapshot_value_to_check_tests; the RIT crates/cli/tests/cli_rit/run_restore_repeat_tests.rs covers it end to
//! end.

use oplog::RefMove;

/// On a repeat (`repeated` true, the moves are a done record's), the move in `moves` whose refname is
/// `snapshot_ref`: its `old` is the value the first attempt checked the folder against, since the restore may already
/// have moved the snapshot ref, and comparing the folder with the ref's restored value would read saved work as
/// unsaved. None on a first attempt, where the restore has moved nothing yet, or when the moves do not
/// include that ref: the snapshot ref's present value is the one to check against.
pub fn snapshot_value_to_check<'a>(repeated: bool, snapshot_ref: &str, moves: &'a [RefMove]) -> Option<&'a RefMove> {
    if !repeated {
        return None;
    }
    moves.iter().find(|m| m.refname == snapshot_ref)
}
