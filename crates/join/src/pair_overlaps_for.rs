//! pair_overlaps_for: one named request's overlapping pairs among requests' changed files. pair_overlaps_for_tests
//! covers each rule.

use crate::OverlapPair;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

/// Merges entries with the same name (the union of their paths), then compares the request named `name` with each
/// other distinct name once (N-1 comparisons, not every pair), giving one OverlapPair for each that shares at least
/// one path, with `first` < `second` by name. Pairs come ordered by (first, second); each pair's paths are listed
/// once, ascending in Path's order (as BTreeSet<PathBuf> keeps them), so `a/b` comes before `a-b`. A name not among
/// the requests gives no pairs; a request with no changed files overlaps nothing. The answer is the same for any
/// order of the input, and equals pair_overlaps' answer kept to the pairs that hold `name`.
pub(crate) fn pair_overlaps_for(changed: &[(String, Vec<PathBuf>)], name: &str) -> Vec<OverlapPair> {
    let mut by_name: BTreeMap<&str, BTreeSet<&PathBuf>> = BTreeMap::new();
    for (request, paths) in changed {
        by_name.entry(request).or_default().extend(paths);
    }
    let Some(named) = by_name.remove(name) else {
        return Vec::new();
    };
    by_name
        .into_iter()
        .filter_map(|(other, other_paths)| {
            let paths: Vec<PathBuf> = named.intersection(&other_paths).map(|p| (*p).clone()).collect();
            if paths.is_empty() {
                return None;
            }
            let (first, second) = if name < other { (name, other) } else { (other, name) };
            Some(OverlapPair { first: first.to_string(), second: second.to_string(), paths })
        })
        .collect()
}
