//! Refuses an op with a move whose ref name is empty or holds a NUL, before apply writes anything.

use crate::store::{OplogError, RefMove};

/// Ok when every move's ref name is usable. The first move whose ref name is empty or holds a NUL gives
/// `BadMove` naming it: gitio refuses such a name in the transaction, and apply checks first so it never
/// leaves an intent open for an op that could not run. Only the names are checked; `old` and `new` are not.
pub fn check_ref_moves(moves: &[RefMove]) -> Result<(), OplogError> {
    for m in moves {
        let reason = if m.refname.is_empty() {
            "is empty"
        } else if m.refname.contains('\0') {
            "holds a NUL"
        } else {
            continue;
        };
        return Err(OplogError::BadMove {
            refname: m.refname.clone(),
            reason: reason.to_string(),
        });
    }
    Ok(())
}
