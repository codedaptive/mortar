//! SaveOutcome: what a save call left behind. A declaration only; no unit test builds it (S-7). Nagatha's save
//! integration tests read it in tests/snapshot/save_cases.rs and save_repair.rs (save's header lists their cases).

use crate::RefusedFile;
use gitio::Oid;
use oplog::ChangeId;

/// The folder's HEAD commit after the call, the saved tree, whether this call wrote that commit and moved the
/// branch to it (false: the tree equalled HEAD's, no commit was written and the branch did not move, the index is
/// set to HEAD's tree, `commit` is HEAD as found and `change_id` is None), the change ID the written commit
/// carries, and the snapshot's refused list passed through: each new file over the size cap left out of the tree,
/// with its size and the cap.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SaveOutcome {
    pub commit: Oid,
    pub tree: Oid,
    pub written: bool,
    pub change_id: Option<ChangeId>,
    pub refused: Vec<RefusedFile>,
}
