//! An op's planned moves as gitio ref updates.

use gitio::RefUpdate;

use crate::store::RefMove;

/// One update per move, in order, each as given: an all-zero `new` stays a delete, `old` None stays
/// "must not exist".
pub(crate) fn ref_updates(moves: &[RefMove]) -> Vec<RefUpdate> {
    moves
        .iter()
        .map(|m| RefUpdate {
            refname: m.refname.clone(),
            new: m.new.clone(),
            old: m.old.clone(),
        })
        .collect()
}
