//! The refs one sweep deletes: closed requests' snapshot refs (attempts' included), replay-record refs of zones the
//! caller no longer lists, joined leftover branches with their snapshot refs, and the
//! caller's extra refs.

use std::collections::HashSet;

use gitio::Oid;
use oplog::ProtectedRef;

use super::{closed_snap_refs, extra_ref_verdict, stale_replayed_refs, ExtraRef, Leftover};
use crate::make::FolderError;
use crate::plan::SNAP_REF_PREFIX;

/// The first of `extra_refs` that extra_ref_verdict refuses is ExtraRefRefused, and nothing is picked. Otherwise
/// (refname, id) pairs, de-duplicated by refname with the first kept, in this order: closed_snap_refs(refs); the
/// replay-record refs stale_replayed_refs picks from `refs` (those of zones not in `zones`); for each of `joined`,
/// its branch at its tip, then its snapshot ref (SNAP_REF_PREFIX + `<zone>/<request>` from the branch name, as
/// closed_snap_refs names it) when `refs` holds it, so both go in the same op; then the accepted extras.
/// A joined leftover's attempt snapshot refs (TRYSNAP_REF_PREFIX) are not in that op: closed_snap_refs keeps them
/// while `refs` holds the request branch or an attempt branch `refs/heads/try/<zone>/<request>/<k>` (k an attempt
/// number), and picks them in a later sweep_refs once neither remains.
pub fn sweep_pick(
    refs: &[(Oid, String)],
    zones: &[String],
    joined: &[Leftover],
    extra_refs: &[String],
    protected: &[ProtectedRef],
) -> Result<Vec<(String, Oid)>, FolderError> {
    let mut accepted = Vec::new();
    for refname in extra_refs {
        match extra_ref_verdict(refname, refs, protected) {
            ExtraRef::Refuse(reason) => {
                return Err(FolderError::ExtraRefRefused { refname: refname.clone(), reason });
            }
            ExtraRef::Accept(oid) => accepted.push((refname.clone(), oid)),
            ExtraRef::Skip => {}
        }
    }
    let mut picked = closed_snap_refs(refs);
    picked.extend(stale_replayed_refs(refs, zones));
    for leftover in joined {
        picked.push((leftover.branch.clone(), leftover.tip.clone()));
        picked.extend(snapshot_of(&leftover.branch, refs));
    }
    picked.extend(accepted);
    let mut seen = HashSet::new();
    picked.retain(|(refname, _)| seen.insert(refname.clone()));
    Ok(picked)
}

/// The snapshot ref of `refs/heads/req/<zone>/<request>` (leftover_branches' shape) with its id, when `refs`
/// holds it.
fn snapshot_of(branch: &str, refs: &[(Oid, String)]) -> Option<(String, Oid)> {
    let zone_request = branch.strip_prefix("refs/heads/req/")?;
    let name = format!("{SNAP_REF_PREFIX}{zone_request}");
    let (oid, _) = refs.iter().find(|(_, other)| *other == name)?;
    Some((name, oid.clone()))
}
