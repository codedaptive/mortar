//! pair_overlaps: each pairing rule, as data with no git.

use crate::pair_overlaps::pair_overlaps;
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
fn no_requests_give_no_pairs() {
    assert_eq!(pair_overlaps(&[]), vec![]);
}

#[test]
fn one_request_gives_no_pairs() {
    assert_eq!(pair_overlaps(&[req("a", &["x", "y"])]), vec![]);
}

#[test]
fn two_disjoint_requests_give_no_pairs() {
    assert_eq!(pair_overlaps(&[req("a", &["x"]), req("b", &["y"])]), vec![]);
}

#[test]
fn two_requests_sharing_one_path_give_one_pair_with_that_path() {
    assert_eq!(
        pair_overlaps(&[req("a", &["x", "only-a"]), req("b", &["only-b", "x"])]),
        vec![pair("a", "b", &["x"])]
    );
}

#[test]
fn of_three_requests_only_the_overlapping_pair_is_listed() {
    assert_eq!(
        pair_overlaps(&[req("a", &["x", "p"]), req("b", &["x", "q"]), req("c", &["r"])]),
        vec![pair("a", "b", &["x"])]
    );
}

#[test]
fn a_path_changed_by_all_three_gives_three_pairs_in_name_order() {
    assert_eq!(
        pair_overlaps(&[req("a", &["x"]), req("b", &["x"]), req("c", &["x"])]),
        vec![pair("a", "b", &["x"]), pair("a", "c", &["x"]), pair("b", "c", &["x"])]
    );
}

#[test]
fn a_request_with_no_changed_files_never_appears() {
    assert_eq!(
        pair_overlaps(&[req("a", &["x"]), req("empty", &[]), req("b", &["x"])]),
        vec![pair("a", "b", &["x"])]
    );
}

#[test]
fn shared_paths_come_out_ascending_and_each_once() {
    assert_eq!(
        pair_overlaps(&[req("a", &["z", "m", "z", "b/c"]), req("b", &["m", "b/c", "z", "m"])]),
        vec![pair("a", "b", &["b/c", "m", "z"])]
    );
}

#[test]
fn a_slash_path_comes_before_a_dash_path_in_path_order() {
    assert_eq!(
        pair_overlaps(&[req("a", &["a-b", "a/b"]), req("b", &["a-b", "a/b"])]),
        vec![pair("a", "b", &["a/b", "a-b"])]
    );
}

#[test]
fn input_in_reverse_name_order_gives_the_same_pairs_first_before_second() {
    assert_eq!(
        pair_overlaps(&[req("c", &["x"]), req("b", &["x", "y"]), req("a", &["y"])]),
        vec![pair("a", "b", &["y"]), pair("b", "c", &["x"])]
    );
}

#[test]
fn a_repeated_name_is_merged_and_never_paired_with_itself() {
    assert_eq!(
        pair_overlaps(&[req("a", &["x", "z"]), req("a", &["y", "z"]), req("b", &["x", "y"])]),
        vec![pair("a", "b", &["x", "y"])]
    );
}

#[test]
fn every_order_of_the_input_gives_the_same_answer() {
    let input = [req("a", &["x", "y"]), req("b", &["y", "z"]), req("c", &["x", "z"]), req("d", &[])];
    let expected = vec![pair("a", "b", &["y"]), pair("a", "c", &["x"]), pair("b", "c", &["z"])];
    let orders: [[usize; 4]; 6] = [[0, 1, 2, 3], [3, 2, 1, 0], [1, 3, 0, 2], [2, 0, 3, 1], [3, 0, 2, 1], [1, 2, 3, 0]];
    for order in orders {
        let shuffled: Vec<_> = order.iter().map(|&i| input[i].clone()).collect();
        assert_eq!(pair_overlaps(&shuffled), expected, "order {order:?}");
    }
}
