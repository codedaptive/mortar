//! pair_overlaps_for: each rule for one named request's pairs, as data with no git.

use crate::pair_overlaps::pair_overlaps;
use crate::pair_overlaps_for::pair_overlaps_for;
use crate::OverlapPair;
use std::path::PathBuf;

fn req(name: &str, paths: &[&str]) -> (String, Vec<PathBuf>) {
    (name.to_string(), paths.iter().map(PathBuf::from).collect())
}

fn pair(first: &str, second: &str, paths: &[&str]) -> OverlapPair {
    OverlapPair {
        first: first.to_string(),
        second: second.to_string(),
        paths: paths.iter().map(PathBuf::from).collect(),
    }
}

#[test]
fn one_overlap_gives_that_pair_with_the_shared_path() {
    assert_eq!(
        pair_overlaps_for(&[req("a", &["x", "only-a"]), req("b", &["only-b", "x"])], "a"),
        vec![pair("a", "b", &["x"])]
    );
}

#[test]
fn no_overlap_gives_no_pairs() {
    assert_eq!(pair_overlaps_for(&[req("a", &["x"]), req("b", &["y"])], "a"), vec![]);
}

#[test]
fn a_named_request_sorting_first_is_the_first_of_its_pair() {
    assert_eq!(
        pair_overlaps_for(&[req("m", &["x"]), req("z", &["x"])], "m"),
        vec![pair("m", "z", &["x"])]
    );
}

#[test]
fn a_named_request_sorting_second_is_the_second_of_its_pair() {
    assert_eq!(
        pair_overlaps_for(&[req("m", &["x"]), req("b", &["x"])], "m"),
        vec![pair("b", "m", &["x"])]
    );
}

#[test]
fn several_others_give_only_the_named_requests_pairs_in_order() {
    let changed = [
        req("e", &["p", "q"]),
        req("m", &["p", "q", "r", "s"]),
        req("a", &["q", "t"]),
        req("z", &["s", "r"]),
        req("c", &["u"]),
        req("y", &["v"]),
        req("b", &["t", "u", "v"]),
    ];
    assert_eq!(
        pair_overlaps_for(&changed, "m"),
        vec![pair("a", "m", &["q"]), pair("e", "m", &["p", "q"]), pair("m", "z", &["r", "s"])]
    );
}

#[test]
fn repeated_entries_of_the_named_request_are_merged() {
    assert_eq!(
        pair_overlaps_for(&[req("a", &["x", "z"]), req("a", &["y", "z"]), req("b", &["x", "y"])], "a"),
        vec![pair("a", "b", &["x", "y"])]
    );
}

#[test]
fn repeated_entries_of_another_request_are_merged_into_one_pair() {
    assert_eq!(
        pair_overlaps_for(&[req("b", &["x"]), req("a", &["x", "y", "w"]), req("b", &["y", "z"])], "a"),
        vec![pair("a", "b", &["x", "y"])]
    );
}

#[test]
fn a_name_not_among_the_requests_gives_no_pairs() {
    assert_eq!(pair_overlaps_for(&[req("a", &["x"]), req("b", &["x"])], "c"), vec![]);
}

#[test]
fn no_requests_give_no_pairs() {
    assert_eq!(pair_overlaps_for(&[], "a"), vec![]);
}

#[test]
fn a_named_request_with_no_changed_files_gives_no_pairs() {
    assert_eq!(pair_overlaps_for(&[req("a", &[]), req("b", &["x"]), req("c", &["x"])], "a"), vec![]);
}

#[test]
fn another_request_with_no_changed_files_never_appears() {
    assert_eq!(
        pair_overlaps_for(&[req("a", &["x"]), req("empty", &[]), req("b", &["x"])], "b"),
        vec![pair("a", "b", &["x"])]
    );
}

#[test]
fn shared_paths_come_out_once_each_in_path_order_slash_before_dash() {
    assert_eq!(
        pair_overlaps_for(&[req("a", &["z", "a-b", "a/b", "z"]), req("b", &["a-b", "z", "a/b", "a-b"])], "b"),
        vec![pair("a", "b", &["a/b", "a-b", "z"])]
    );
}

#[test]
fn every_order_of_the_input_gives_the_same_answer() {
    let input = [req("c", &["x", "z"]), req("a", &["x", "y"]), req("d", &[]), req("b", &["y", "z"]), req("c", &["w"])];
    let expected = vec![pair("a", "c", &["x"]), pair("b", "c", &["z"])];
    let orders: [[usize; 5]; 6] =
        [[0, 1, 2, 3, 4], [4, 3, 2, 1, 0], [1, 3, 0, 4, 2], [2, 4, 3, 1, 0], [3, 0, 4, 2, 1], [1, 2, 4, 0, 3]];
    for order in orders {
        let shuffled: Vec<_> = order.iter().map(|&i| input[i].clone()).collect();
        assert_eq!(pair_overlaps_for(&shuffled, "c"), expected, "order {order:?}");
    }
}

#[test]
fn the_answer_is_pair_overlaps_filtered_to_the_named_request() {
    let inputs = [
        vec![req("a", &["x", "y"]), req("b", &["y", "z"]), req("c", &["x", "z"]), req("d", &[])],
        vec![req("c", &["x"]), req("b", &["x", "y"]), req("a", &["y"]), req("b", &["q"]), req("e", &["q", "x"])],
        vec![req("m", &["a/b", "a-b"]), req("k", &["a-b"]), req("n", &["a/b"]), req("k", &["c"]), req("n", &["c"])],
    ];
    for changed in &inputs {
        for (name, _) in changed {
            let expected: Vec<OverlapPair> = pair_overlaps(changed)
                .into_iter()
                .filter(|p| p.first == *name || p.second == *name)
                .collect();
            assert_eq!(pair_overlaps_for(changed, name), expected, "name {name} in {changed:?}");
        }
    }
}
