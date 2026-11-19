//! open_requests_for: a zone's open requests as join's overlap takes them.

use ::join::OpenRequest;
use folders::{FolderKind, FolderRecord};
use gitio::Oid;

/// For each folder as (its record, its request branch's tip, its snapshot ref's commit), in the order given: a
/// Request folder of `zone` naming a request whose branch exists is open. Its tip is the snapshot commit when it
/// has one, else the branch tip (each one's tip, or its last snapshot when it has one); its base is
/// `fork_point(name, &tip)`, the commit it shares with the zone, never the record's base, which is the cut
/// commit and goes stale when the request is rebased. Zone and attempt folders, other zones' folders and requests
/// whose branch is gone are skipped, and fork_point is called for none of them. The first error from fork_point is
/// returned.
pub fn open_requests_for<E>(
    zone: &str,
    folders: &[(FolderRecord, Option<Oid>, Option<Oid>)],
    mut fork_point: impl FnMut(&str, &Oid) -> Result<Oid, E>,
) -> Result<Vec<OpenRequest>, E> {
    folders
        .iter()
        .filter(|(record, ..)| record.kind == FolderKind::Request && record.zone == zone)
        .filter_map(|(record, branch_tip, snapshot)| {
            Some((record.request.clone()?, snapshot.as_ref().unwrap_or(branch_tip.as_ref()?).clone()))
        })
        .map(|(name, tip)| Ok(OpenRequest { base: fork_point(&name, &tip)?, name, tip }))
        .collect()
}
