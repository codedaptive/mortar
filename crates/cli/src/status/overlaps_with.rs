//! overlaps_with: one request's overlaps, from join's pairs for its whole zone.

use std::path::PathBuf;

use ::join::OverlapPair;

/// Each pair naming `request` first or second, as (the other request's name, the shared paths), in pair order.
pub fn overlaps_with(request: &str, pairs: &[OverlapPair]) -> Vec<(String, Vec<PathBuf>)> {
    pairs
        .iter()
        .filter_map(|pair| match (pair.first == request, pair.second == request) {
            (true, _) => Some((pair.second.clone(), pair.paths.clone())),
            (_, true) => Some((pair.first.clone(), pair.paths.clone())),
            _ => None,
        })
        .collect()
}
