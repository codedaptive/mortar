use crate::cap_candidate::CapCandidate;
use crate::cap_refusals::cap_refusals;
use crate::default_size_cap::DEFAULT_SIZE_CAP;
use crate::refused_file::RefusedFile;
use std::path::PathBuf;

fn candidate(path: &str, size: u64, in_head: bool) -> CapCandidate {
    CapCandidate { path: PathBuf::from(path), size, in_head }
}

#[test]
fn a_new_file_over_the_cap_is_refused_with_its_size_and_the_cap() {
    assert_eq!(
        cap_refusals(&[candidate("big.bin", 101, false)], 100),
        vec![RefusedFile { path: PathBuf::from("big.bin"), size: 101, cap: 100 }]
    );
}

#[test]
fn a_new_file_of_exactly_the_cap_is_kept() {
    assert_eq!(cap_refusals(&[candidate("edge.bin", 100, false)], 100), vec![]);
}

#[test]
fn a_new_file_under_the_cap_is_kept() {
    assert_eq!(cap_refusals(&[candidate("small.txt", 99, false)], 100), vec![]);
}

#[test]
fn a_file_in_head_over_the_cap_is_kept() {
    assert_eq!(cap_refusals(&[candidate("old.bin", 500, true)], 100), vec![]);
}

#[test]
fn only_the_refused_are_listed_in_input_order() {
    let candidates = [
        candidate("z.bin", 300, false),
        candidate("a.txt", 10, false),
        candidate("m.bin", 200, true),
        candidate("b.bin", 150, false),
    ];
    assert_eq!(
        cap_refusals(&candidates, 100),
        vec![
            RefusedFile { path: PathBuf::from("z.bin"), size: 300, cap: 100 },
            RefusedFile { path: PathBuf::from("b.bin"), size: 150, cap: 100 },
        ]
    );
}

#[test]
fn no_candidates_gives_no_refusals() {
    assert_eq!(cap_refusals(&[], 100), vec![]);
}

#[test]
fn the_default_cap_refuses_a_new_file_of_5_242_881_bytes() {
    assert_eq!(
        cap_refusals(&[candidate("six.bin", 5_242_881, false)], DEFAULT_SIZE_CAP),
        vec![RefusedFile { path: PathBuf::from("six.bin"), size: 5_242_881, cap: 5_242_880 }]
    );
}

#[test]
fn the_default_cap_keeps_a_new_file_of_5_242_880_bytes() {
    assert_eq!(cap_refusals(&[candidate("five.bin", 5_242_880, false)], DEFAULT_SIZE_CAP), vec![]);
}
