//! Which snapshot refs, a request's and its attempts', belong to closed requests, so sweep_refs may delete them. A
//! closed request is one whose request branch and attempt branches are all gone. keep leaves no attempt branch; remove
//! deletes only the request branch and its snapshot ref and leaves any attempt branches, so such a request stays open.
//! A request branch left behind is leftover_branches' case (S-52 (1)); t3code's checkpoint refs come in as
//! the caller's extra refs (S-53, S-54).

use gitio::Oid;

use crate::plan::{SNAP_REF_PREFIX, TRYSNAP_REF_PREFIX};

/// The snapshot refs in `refs` (list_refs' shape: id, then full refname) of closed requests, as (refname, id), in
/// input order. Only a ref named SNAP_REF_PREFIX + `<zone>/<request>` (two non-empty segments) or
/// TRYSNAP_REF_PREFIX + `<zone>/<request>/<k>` (three non-empty segments) is considered; any other is left alone.
/// Its request is open while `refs` holds `refs/heads/req/<zone>/<request>` or any ref starting with
/// `refs/heads/try/<zone>/<request>/` (an attempt branch, S-69), and closed otherwise.
pub fn closed_snap_refs(refs: &[(Oid, String)]) -> Vec<(String, Oid)> {
    refs.iter()
        .filter(|(_, name)| match zone_request(name) {
            Some((zone, request)) => !is_open(refs, zone, request),
            None => false,
        })
        .map(|(oid, name)| (name.clone(), oid.clone()))
        .collect()
}

fn zone_request(name: &str) -> Option<(&str, &str)> {
    if let Some(rest) = name.strip_prefix(SNAP_REF_PREFIX) {
        return segments(rest);
    }
    let (zone_request, k) = name.strip_prefix(TRYSNAP_REF_PREFIX)?.rsplit_once('/')?;
    if k.is_empty() {
        return None;
    }
    segments(zone_request)
}

/// `<zone>/<request>`, both non-empty, with no further '/'.
fn segments(text: &str) -> Option<(&str, &str)> {
    let (zone, request) = text.split_once('/')?;
    if zone.is_empty() || request.is_empty() || request.contains('/') {
        return None;
    }
    Some((zone, request))
}

fn is_open(refs: &[(Oid, String)], zone: &str, request: &str) -> bool {
    let request_branch = format!("refs/heads/req/{zone}/{request}");
    let attempt_prefix = format!("refs/heads/try/{zone}/{request}/");
    refs.iter()
        .any(|(_, name)| *name == request_branch || name.starts_with(&attempt_prefix))
}
