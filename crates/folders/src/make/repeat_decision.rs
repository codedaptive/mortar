//! Is the folder record found at a path the one a make call wants?

use super::Repeat;
use crate::record::FolderRecord;

/// Same when every field but `base` and `needed_paths` is equal, else Different. `base` is left out because a
/// zone branch that moved since the folder was cut does not make it another folder; `needed_paths` because
/// need() widens a folder after it was made, and the repeat's refill keeps those paths (refill_cone). On Same
/// the make call refills the folder (unless its index has unmerged entries) and answers its record, except that
/// make_request_folder_on first refuses another recorded base or other partners (other_base_refusal), unless the
/// call asks for no partners at the zone tip and the folder's record holds no partners, which refills as
/// make_request_folder does; on Different it refuses.
pub fn repeat_decision(wanted: &FolderRecord, existing: &FolderRecord) -> Repeat {
    let same = wanted.kind == existing.kind
        && wanted.zone == existing.zone
        && wanted.request == existing.request
        && wanted.work_order_paths == existing.work_order_paths
        && wanted.parent == existing.parent;
    if same {
        Repeat::Same
    } else {
        Repeat::Different
    }
}
