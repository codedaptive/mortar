//! refs_by_namespace: four or more segments count under the first three, two or three under the first two, one
//! under itself; counts aggregate; output is sorted by namespace.

use gitio::Oid;

use super::refs_by_namespace::refs_by_namespace;

const ID: &str = "0123456789abcdef0123456789abcdef01234567";

fn refs(names: &[&str]) -> Vec<(Oid, String)> {
    names.iter().map(|name| (Oid::parse(ID).unwrap(), name.to_string())).collect()
}

fn one(name: &str) -> Vec<(String, usize)> {
    refs_by_namespace(&refs(&[name]))
}

#[test]
fn three_segments_count_under_the_first_two() {
    assert_eq!(one("refs/heads/main"), vec![("refs/heads".to_string(), 1)]);
    assert_eq!(one("refs/tags/v1"), vec![("refs/tags".to_string(), 1)]);
}

#[test]
fn four_segments_count_under_the_first_three() {
    assert_eq!(one("refs/heads/zone/folders"), vec![("refs/heads/zone".to_string(), 1)]);
}

#[test]
fn five_segments_count_under_the_first_three() {
    assert_eq!(one("refs/heads/req/folders/WO-1"), vec![("refs/heads/req".to_string(), 1)]);
    assert_eq!(one("refs/mortar/snap/folders/WO-1"), vec![("refs/mortar/snap".to_string(), 1)]);
}

#[test]
fn two_segments_are_their_own_namespace() {
    assert_eq!(one("refs/stash"), vec![("refs/stash".to_string(), 1)]);
}

#[test]
fn one_segment_is_its_own_namespace() {
    assert_eq!(one("refs"), vec![("refs".to_string(), 1)]);
}

#[test]
fn no_refs_give_no_namespaces() {
    assert_eq!(refs_by_namespace(&[]), Vec::<(String, usize)>::new());
}

#[test]
fn counts_aggregate_and_sort_by_namespace() {
    let input = refs(&[
        "refs/tags/v1",
        "refs/mortar/snap/folders/WO-1",
        "refs/heads/main",
        "refs/mortar/snap/join/WO-2",
        "refs/heads/dev",
    ]);
    assert_eq!(
        refs_by_namespace(&input),
        vec![
            ("refs/heads".to_string(), 2),
            ("refs/mortar/snap".to_string(), 2),
            ("refs/tags".to_string(), 1),
        ]
    );
}
