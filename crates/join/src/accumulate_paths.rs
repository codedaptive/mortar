//! accumulate_paths: the conflicted paths of a join over all its rounds. accumulate_paths_tests covers an empty
//! earlier list, disjoint and overlapping lists, and a repeat within a round.

use std::path::PathBuf;

/// `earlier`'s paths in order, as given, then each of `round`'s paths not already in the result, in order, so a
/// path in `round` is listed once, even when it repeats within `round`.
pub(crate) fn accumulate_paths(earlier: &[PathBuf], round: &[PathBuf]) -> Vec<PathBuf> {
    let mut all = earlier.to_vec();
    for path in round {
        if !all.contains(path) {
            all.push(path.clone());
        }
    }
    all
}
