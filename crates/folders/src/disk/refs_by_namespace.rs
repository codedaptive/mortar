//! How many refs each namespace holds, for the disk report.

use std::collections::BTreeMap;

use gitio::Oid;

/// Counts `refs` (gitio's list_refs shape: id, then full refname) by namespace, sorted by namespace. A name with
/// four or more '/'-segments counts under its first three (`refs/heads/zone/folders` under `refs/heads/zone`), one
/// with two or three under its first two (`refs/tags/v1` under `refs/tags`), and one with a single segment under
/// itself.
pub fn refs_by_namespace(refs: &[(Oid, String)]) -> Vec<(String, usize)> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for (_, name) in refs {
        *counts.entry(namespace(name)).or_insert(0) += 1;
    }
    counts.into_iter().collect()
}

fn namespace(name: &str) -> String {
    let segments: Vec<&str> = name.split('/').collect();
    let keep = match segments.len() {
        n if n >= 4 => 3,
        n if n >= 2 => 2,
        _ => 1,
    };
    segments[..keep].join("/")
}
