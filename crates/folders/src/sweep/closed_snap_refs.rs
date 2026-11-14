//! Which snapshot refs, a request's and its attempts', belong to closed requests, so sweep_refs may delete them. A
//! closed request is one whose request branch and every attempt branch (`refs/heads/try/<zone>/<request>/<k>` with
//! attempt_number(k) Some, S-69) are gone. keep and drop_attempts delete every attempt branch; remove refuses
//! (AttemptsRemain) while an attempt branch or attempt folder remains, so a request with an attempt branch has not
//! been removed and stays open. Any other branch under that prefix (`01`, `0`, `1/x`) is not Mortar's: it does not
//! keep a request open, and the sweep picks only snap and trysnap refs, so that branch is never touched.
//! A request branch left behind is leftover_branches' case (S-52 (1)); t3code's checkpoint refs come in as
//! the caller's extra refs (S-53, S-54).

use gitio::Oid;

use crate::plan::{attempt_number, SNAP_REF_PREFIX, TRYSNAP_REF_PREFIX};

/// The snapshot refs in `refs` (list_refs' shape: id, then full refname) of closed requests, as (refname, id), in
/// input order. Only a ref named SNAP_REF_PREFIX + `<zone>/<request>` (two non-empty segments) or
/// TRYSNAP_REF_PREFIX + `<zone>/<request>/<k>` (three non-empty segments) is considered; any other is left alone.
/// Its request is open while `refs` holds `refs/heads/req/<zone>/<request>` or an attempt branch, a ref named
/// exactly `refs/heads/try/<zone>/<request>/<k>` with attempt_number(k) Some (S-69), and closed otherwise.
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
    refs.iter().any(|(_, name)| {
        *name == request_branch || name.strip_prefix(&attempt_prefix).and_then(attempt_number).is_some()
    })
}
