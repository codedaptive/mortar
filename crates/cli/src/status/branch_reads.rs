//! BranchReads: what read_branch_reads reads about a request or attempt folder's branch, its zone and its tree.
//! A bare type; joined_from_reads_tests builds it.

use gitio::Oid;
use snapshot::SavedState;

/// The folder's branch as a full ref (`branch_ref`) and its tip (`branch_tip`); the zone branch's tip (`zone_tip`);
/// whether the zone holds the branch tip (`request_in_zone`); saved_state's answer for the folder (`saved`, whose
/// tree is the folder's tree); and the branch tip's tree (`tip_tree`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchReads {
    pub branch_ref: String,
    pub branch_tip: Oid,
    pub zone_tip: Oid,
    pub request_in_zone: bool,
    pub saved: SavedState,
    pub tip_tree: Oid,
}
