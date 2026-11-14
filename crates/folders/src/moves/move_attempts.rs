//! The tries move_folder makes at a folder's two-tree move, in order (S-88).

use gitio::Oid;

use super::MoveAttempt;

/// Without `index_tree`: one plain move from `old`. With `index_tree` t: the plain move from `old` first, then, for
/// when git refuses it, set the index to t and move from t. The plain move comes first because a folder already at
/// the new tip moves as a no-op, where setting the index to t would put it out of step with the files (S-88).
/// t equal to `old` still gives both tries: the folder's index need not be at `old`, so setting it can matter.
pub(crate) fn move_attempts<'a>(old: &'a Oid, index_tree: Option<&'a Oid>) -> Vec<MoveAttempt<'a>> {
    let plain = MoveAttempt { set_index: None, from: old };
    match index_tree {
        None => vec![plain],
        Some(t) => vec![plain, MoveAttempt { set_index: Some(t), from: t }],
    }
}
