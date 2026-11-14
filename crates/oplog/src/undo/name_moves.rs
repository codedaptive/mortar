//! Ref moves built from bare ref names, so restore can run apply's ref name checks before it reads any ref.

use gitio::Oid;

use crate::store::{OplogError, RefMove};

/// One move per name in `refs`, in order, each with old None and new 40 zeros. check_ref_moves and
/// check_ref_names read only a move's ref name, so the values are placeholders: these moves are never applied.
pub(crate) fn name_moves(refs: &[String]) -> Result<Vec<RefMove>, OplogError> {
    let zero = Oid::parse("0000000000000000000000000000000000000000").map_err(OplogError::Git)?;
    Ok(refs
        .iter()
        .map(|r| RefMove {
            refname: r.clone(),
            old: None,
            new: zero.clone(),
        })
        .collect())
}
