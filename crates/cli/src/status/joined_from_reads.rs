//! joined_from_reads: whether a request is joined to its zone, from read_branch_reads' answer.

use gitio::Oid;

use super::{is_joined, BranchReads};

/// is_joined fed as status feeds it: the record's `base`, the branch tip, whether the zone holds it, the folder's
/// tree as saved_state built it and the tip's tree.
pub fn joined_from_reads(base: &Oid, reads: &BranchReads) -> bool {
    is_joined(base, &reads.branch_tip, reads.request_in_zone, &reads.saved.tree, &reads.tip_tree)
}
