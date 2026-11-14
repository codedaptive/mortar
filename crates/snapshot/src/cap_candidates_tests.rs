use crate::cap_candidate::CapCandidate;
use crate::cap_candidates::cap_candidates;
use std::collections::HashSet;
use std::path::PathBuf;

fn sized() -> Vec<(PathBuf, u64)> {
    vec![(PathBuf::from("c.bin"), 30), (PathBuf::from("a.txt"), 10), (PathBuf::from("b.rs"), 20)]
}

#[test]
fn without_head_paths_every_candidate_is_not_in_head() {
    assert_eq!(
        cap_candidates(&sized(), None),
        vec![
            CapCandidate { path: PathBuf::from("c.bin"), size: 30, in_head: false },
            CapCandidate { path: PathBuf::from("a.txt"), size: 10, in_head: false },
            CapCandidate { path: PathBuf::from("b.rs"), size: 20, in_head: false },
        ]
    );
}

#[test]
fn with_head_paths_exactly_the_held_paths_are_in_head() {
    let head: HashSet<PathBuf> = [PathBuf::from("a.txt"), PathBuf::from("other.md")].into_iter().collect();
    assert_eq!(
        cap_candidates(&sized(), Some(&head)),
        vec![
            CapCandidate { path: PathBuf::from("c.bin"), size: 30, in_head: false },
            CapCandidate { path: PathBuf::from("a.txt"), size: 10, in_head: true },
            CapCandidate { path: PathBuf::from("b.rs"), size: 20, in_head: false },
        ]
    );
}

#[test]
fn the_sized_order_is_kept() {
    let paths: Vec<PathBuf> = cap_candidates(&sized(), None).into_iter().map(|c| c.path).collect();
    assert_eq!(paths, vec![PathBuf::from("c.bin"), PathBuf::from("a.txt"), PathBuf::from("b.rs")]);
}
