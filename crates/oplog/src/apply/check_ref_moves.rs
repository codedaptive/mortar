//! Refuses an op with a move whose ref name is empty, holds a NUL or is not a full refs/ name, before apply
//! writes anything.

use crate::store::{OplogError, RefMove};

/// Ok when every move's ref name is usable. The first move whose ref name is empty, holds a NUL or does not
/// start with `refs/` gives `BadMove` naming it. A full name is required because apply moves refs and
/// read_ref_value reads them by full name: git check-ref-format accepts a two-level name such as
/// `heads/main`, which update-ref would write outside refs/ and read_ref_value could never read back. Only the
/// names are checked; `old` and `new` are not.
pub(crate) fn check_ref_moves(moves: &[RefMove]) -> Result<(), OplogError> {
    for m in moves {
        let reason = if m.refname.is_empty() {
            "is empty"
        } else if m.refname.contains('\0') {
            "holds a NUL"
        } else if !m.refname.starts_with("refs/") {
            "is not a full ref name (it must start with refs/)"
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
