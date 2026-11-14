//! Which request branches are checked out nowhere and have a zone branch to be joined to, so sweep_refs may test
//! them for containment (S-52 (1)).

use gitio::Oid;

use super::Leftover;

const REQ_PREFIX: &str = "refs/heads/req/";

/// The refs in `refs` (list_refs' shape: id, then full refname), in input order, named exactly
/// `refs/heads/req/<zone>/<request>` (two non-empty segments: the request branch's shape), not in `checked_out`
/// (full branch names, as gitio's Worktree.branch gives them), and whose `refs/heads/zone/<zone>` is in `refs`; that
/// ref's id is the zone_tip.
pub fn leftover_branches(refs: &[(Oid, String)], checked_out: &[String]) -> Vec<Leftover> {
    refs.iter()
        .filter(|(_, name)| !checked_out.contains(name))
        .filter_map(|(tip, name)| {
            let zone_branch = format!("refs/heads/zone/{}", zone_of(name)?);
            let (zone_tip, _) = refs.iter().find(|(_, other)| *other == zone_branch)?;
            Some(Leftover {
                branch: name.clone(),
                tip: tip.clone(),
                zone_tip: zone_tip.clone(),
            })
        })
        .collect()
}

fn zone_of(name: &str) -> Option<&str> {
    let (zone, request) = name.strip_prefix(REQ_PREFIX)?.split_once('/')?;
    if zone.is_empty() || request.is_empty() || request.contains('/') {
        return None;
    }
    Some(zone)
}
