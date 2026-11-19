//! PriorSquash: the commit already at a zone's squash ref. A declaration only; choose_change_id_tests
//! exercises it.

use gitio::{CommitInfo, Oid};
use oplog::ChangeId;

/// The commit, what it holds, its Mortar-Change-Id (None: it has none) and whether it is an ancestor of origin
/// (already landed, so the next squash is a new change).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PriorSquash {
    pub(crate) commit: Oid,
    pub(crate) info: CommitInfo,
    pub(crate) change_id: Option<ChangeId>,
    pub(crate) in_origin: bool,
}
