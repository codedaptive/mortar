//! own_refs: the refs a folder's undo may reach, its own branch and its own snapshot ref (S-142 item 2).

use folders::plan::{attempt_branch, request_branch, snap_ref, TRYSNAP_REF_PREFIX};
use folders::{FolderKind, FolderRecord};

/// Full ref names, the branch first: refs/heads/req/<zone>/<request> and refs/mortar/snap/<zone>/<request> for a
/// request folder; refs/heads/try/<zone>/<request>/<k> and refs/mortar/trysnap/<zone>/<request>/<k> for attempt
/// folder k. None for a zone folder, a record naming no request, or names folders' plan refuses.
pub fn own_refs(record: &FolderRecord) -> Option<Vec<String>> {
    let (zone, request) = (&record.zone, record.request.as_deref()?);
    match record.kind {
        FolderKind::Request => {
            Some(vec![format!("refs/heads/{}", request_branch(zone, request).ok()?), snap_ref(zone, request).ok()?])
        }
        FolderKind::Attempt(k) => Some(vec![
            format!("refs/heads/{}", attempt_branch(zone, request, k).ok()?),
            format!("{TRYSNAP_REF_PREFIX}{zone}/{request}/{k}"),
        ]),
        FolderKind::Zone => None,
    }
}
