//! pair_overlaps: the overlapping pairs among requests' changed files. pair_overlaps_tests covers each rule.

use crate::OverlapPair;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

/// Merges entries with the same name (the union of their paths), then gives one OverlapPair for each pair of
/// distinct names whose paths share at least one path, with `first` < `second`. Pairs come ordered by (first,
/// second); each pair's paths are listed once, ascending in Path's order, which compares component by component
/// (as BTreeSet<PathBuf> keeps them), so `a/b` comes before `a-b`, unlike byte order. A request with no changed
/// files overlaps nothing. The answer is the same for any order of the input.
pub(crate) fn pair_overlaps(changed: &[(String, Vec<PathBuf>)]) -> Vec<OverlapPair> {
    let mut by_name: BTreeMap<&str, BTreeSet<&PathBuf>> = BTreeMap::new();
    for (name, paths) in changed {
        by_name.entry(name).or_default().extend(paths);
    }
    let requests: Vec<_> = by_name.into_iter().collect();
    let mut pairs = Vec::new();
    for (i, (first, first_paths)) in requests.iter().enumerate() {
        for (second, second_paths) in &requests[i + 1..] {
            let paths: Vec<PathBuf> = first_paths.intersection(second_paths).map(|p| (*p).clone()).collect();
            if !paths.is_empty() {
                pairs.push(OverlapPair { first: first.to_string(), second: second.to_string(), paths });
            }
        }
    }
    pairs
}
