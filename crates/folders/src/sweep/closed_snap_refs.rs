//! Which snapshot refs belong to closed requests, so sweep_refs may delete them. A closed request is one whose
//! request branch is gone, which is what remove leaves. A request branch left behind is leftover_branches' case
//! (S-52 (1)); t3code's checkpoint refs come in as the caller's extra refs (S-53, S-54).

use gitio::Oid;

use crate::plan::SNAP_REF_PREFIX;

/// The snapshot refs in `refs` (list_refs' shape: id, then full refname) of closed requests, as (refname, id), in
/// input order. Only a ref named SNAP_REF_PREFIX + `<zone>/<request>`, exactly two non-empty segments, is
/// considered; any other is left alone. Its request is closed when `refs` holds neither
/// `refs/heads/req/<zone>/<request>` nor any ref starting with `refs/heads/req/<zone>/<request>/` (an attempt
/// branch).
pub fn closed_snap_refs(refs: &[(Oid, String)]) -> Vec<(String, Oid)> {
    refs.iter()
        .filter(|(_, name)| match zone_request(name) {
            Some((zone, request)) => !is_open(refs, &format!("refs/heads/req/{zone}/{request}")),
            None => false,
        })
        .map(|(oid, name)| (name.clone(), oid.clone()))
        .collect()
}

fn zone_request(name: &str) -> Option<(&str, &str)> {
    let (zone, request) = name.strip_prefix(SNAP_REF_PREFIX)?.split_once('/')?;
    if zone.is_empty() || request.is_empty() || request.contains('/') {
        return None;
    }
    Some((zone, request))
}

fn is_open(refs: &[(Oid, String)], branch: &str) -> bool {
    refs.iter().any(|(_, name)| {
        name == branch || name.strip_prefix(branch).is_some_and(|rest| rest.starts_with('/'))
    })
}
