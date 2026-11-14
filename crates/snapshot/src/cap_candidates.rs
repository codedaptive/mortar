//! cap_candidates: the changed files with their sizes, marked by whether HEAD holds them.

use crate::cap_candidate::CapCandidate;
use std::collections::HashSet;
use std::path::PathBuf;

/// One candidate per sized path, in order. `in_head` is whether `head_paths` holds the path; with None (HEAD not
/// listed yet) every candidate is marked not in HEAD, which can only add refusals, never hide one.
pub(crate) fn cap_candidates(sized: &[(PathBuf, u64)], head_paths: Option<&HashSet<PathBuf>>) -> Vec<CapCandidate> {
    sized
        .iter()
        .map(|(path, size)| CapCandidate {
            path: path.clone(),
            size: *size,
            in_head: head_paths.is_some_and(|head| head.contains(path)),
        })
        .collect()
}
