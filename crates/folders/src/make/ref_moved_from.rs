//! The RefMoved refusal an op log record states, when the op was refused because a ref had moved.

use oplog::{OpRecord, OpResult};

use super::FolderError;

/// Some(RefMoved) naming the stale ref and what it held (None: absent) when `record`'s result is Stale; None for
/// any other result, or none.
pub(crate) fn ref_moved_from(record: &OpRecord) -> Option<FolderError> {
    match &record.result {
        Some(OpResult::Stale { refname, actual }) => Some(FolderError::RefMoved {
            refname: refname.clone(),
            at: actual.clone(),
        }),
        _ => None,
    }
}
