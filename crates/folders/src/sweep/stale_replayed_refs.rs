//! Which replay-record refs (`refs/mortar/replayed/<zone>/<source>`) are stale, so sweep_refs may delete them.
//! S-121 (2): a live zone's records are left to the publishing zone's after_publish (land); the sweep removes the
//! family only for removed zones, a zone no longer in mortar's config zones. The caller passes that zone list.
//! Its tests are zone unit tests in stale_replayed_refs_tests: the function is pure and runs no git.

use gitio::Oid;

const REPLAYED_REF_PREFIX: &str = "refs/mortar/replayed/";

/// The replay-record refs in `refs` (list_refs' shape: id, then full refname) whose zone is not in `zones`, as
/// (refname, id), in input order. Only a ref named REPLAYED_REF_PREFIX + `<zone>/<source>` is considered, split at
/// the last '/': `<zone>` non-empty with no empty segment (it may hold '/'), `<source>` non-empty; any other is
/// left alone. `<zone>` is kept when it equals an entry of `zones` exactly, so an empty list picks every one.
pub fn stale_replayed_refs(refs: &[(Oid, String)], zones: &[String]) -> Vec<(String, Oid)> {
    refs.iter()
        .filter(|(_, name)| zone_of(name).is_some_and(|zone| !zones.iter().any(|listed| listed == zone)))
        .map(|(oid, name)| (name.clone(), oid.clone()))
        .collect()
}

fn zone_of(name: &str) -> Option<&str> {
    let (zone, source) = name.strip_prefix(REPLAYED_REF_PREFIX)?.rsplit_once('/')?;
    if source.is_empty() || zone.split('/').any(str::is_empty) {
        return None;
    }
    Some(zone)
}
