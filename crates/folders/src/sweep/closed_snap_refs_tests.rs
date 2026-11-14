//! closed_snap_refs: a closed request's snapshot and attempt snapshot refs are picked; a request branch or an
//! attempt branch keeps them open, by exact name; odd-shaped snapshot refs and other refs are ignored; input order
//! is kept.

use gitio::Oid;

use super::closed_snap_refs::closed_snap_refs;

const A: &str = "0123456789abcdef0123456789abcdef01234567";
const B: &str = "fedcba9876543210fedcba9876543210fedcba98";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn refs(names: &[(&str, &str)]) -> Vec<(Oid, String)> {
    names.iter().map(|(hex, name)| (oid(hex), name.to_string())).collect()
}

#[test]
fn an_open_request_is_kept() {
    let input = refs(&[(A, "refs/heads/req/z/r"), (B, "refs/mortar/snap/z/r")]);
    assert_eq!(closed_snap_refs(&input), Vec::<(String, Oid)>::new());
}

#[test]
fn a_closed_request_is_picked() {
    let input = refs(&[(A, "refs/heads/zone/z"), (B, "refs/mortar/snap/z/r")]);
    assert_eq!(closed_snap_refs(&input), vec![("refs/mortar/snap/z/r".to_string(), oid(B))]);
}

#[test]
fn an_attempt_branch_keeps_it_open() {
    let input = refs(&[(B, "refs/mortar/snap/z/r"), (A, "refs/heads/try/z/r/1")]);
    assert_eq!(closed_snap_refs(&input), Vec::<(String, Oid)>::new());
}

#[test]
fn a_closed_requests_trysnap_ref_is_picked() {
    let input = refs(&[
        (A, "refs/heads/zone/z"),
        (B, "refs/mortar/trysnap/z/r/1"),
        (A, "refs/mortar/snap/z/r"),
        (A, "refs/mortar/trysnap/z/r/2"),
    ]);
    assert_eq!(
        closed_snap_refs(&input),
        vec![
            ("refs/mortar/trysnap/z/r/1".to_string(), oid(B)),
            ("refs/mortar/snap/z/r".to_string(), oid(A)),
            ("refs/mortar/trysnap/z/r/2".to_string(), oid(A)),
        ]
    );
}

#[test]
fn a_request_branch_keeps_snap_and_trysnap_refs() {
    let input = refs(&[(A, "refs/heads/req/z/r"), (B, "refs/mortar/snap/z/r"), (B, "refs/mortar/trysnap/z/r/1")]);
    assert_eq!(closed_snap_refs(&input), Vec::<(String, Oid)>::new());
}

#[test]
fn a_try_branch_keeps_snap_and_trysnap_refs() {
    let input = refs(&[(B, "refs/mortar/snap/z/r"), (B, "refs/mortar/trysnap/z/r/1"), (A, "refs/heads/try/z/r/2")]);
    assert_eq!(closed_snap_refs(&input), Vec::<(String, Oid)>::new());
}

#[test]
fn a_try_branch_of_a_request_sharing_a_prefix_does_not_keep_it_open() {
    let input = refs(&[(A, "refs/heads/try/z/rx/1"), (B, "refs/mortar/trysnap/z/r/1"), (A, "refs/mortar/snap/z/r")]);
    assert_eq!(
        closed_snap_refs(&input),
        vec![("refs/mortar/trysnap/z/r/1".to_string(), oid(B)), ("refs/mortar/snap/z/r".to_string(), oid(A))]
    );
}

#[test]
fn a_ref_nested_under_the_request_branch_name_does_not_keep_it_open() {
    let input = refs(&[(A, "refs/heads/req/z/r/try-1"), (B, "refs/mortar/snap/z/r")]);
    assert_eq!(closed_snap_refs(&input), vec![("refs/mortar/snap/z/r".to_string(), oid(B))]);
}

#[test]
fn an_odd_shaped_trysnap_ref_is_ignored() {
    let input = refs(&[
        (A, "refs/mortar/trysnap/z/r"),
        (A, "refs/mortar/trysnap/z/r/"),
        (A, "refs/mortar/trysnap/z//1"),
        (A, "refs/mortar/trysnap//r/1"),
        (A, "refs/mortar/trysnap/z/r/1/extra"),
        (A, "refs/mortar/trysnap/z"),
    ]);
    assert_eq!(closed_snap_refs(&input), Vec::<(String, Oid)>::new());
}

#[test]
fn a_branch_sharing_a_prefix_does_not_keep_it_open() {
    let input = refs(&[(A, "refs/heads/req/z/rx"), (B, "refs/mortar/snap/z/r")]);
    assert_eq!(closed_snap_refs(&input), vec![("refs/mortar/snap/z/r".to_string(), oid(B))]);
}

#[test]
fn a_same_request_in_another_zone_does_not_keep_it_open() {
    let input = refs(&[(A, "refs/heads/req/y/r"), (B, "refs/mortar/snap/z/r")]);
    assert_eq!(closed_snap_refs(&input), vec![("refs/mortar/snap/z/r".to_string(), oid(B))]);
}

#[test]
fn an_odd_shaped_snap_ref_is_ignored() {
    let input = refs(&[
        (A, "refs/mortar/snap/z"),
        (A, "refs/mortar/snap/z/r/extra"),
        (A, "refs/mortar/snap//r"),
        (A, "refs/mortar/snap/z/"),
    ]);
    assert_eq!(closed_snap_refs(&input), Vec::<(String, Oid)>::new());
}

#[test]
fn a_non_snap_ref_is_ignored() {
    let input = refs(&[(A, "refs/heads/main"), (B, "refs/mortar/other/z/r")]);
    assert_eq!(closed_snap_refs(&input), Vec::<(String, Oid)>::new());
}

#[test]
fn input_order_is_kept() {
    let input = refs(&[
        (B, "refs/mortar/snap/z/b"),
        (A, "refs/heads/req/z/open"),
        (A, "refs/mortar/snap/z/open"),
        (A, "refs/mortar/snap/y/a"),
    ]);
    assert_eq!(
        closed_snap_refs(&input),
        vec![("refs/mortar/snap/z/b".to_string(), oid(B)), ("refs/mortar/snap/y/a".to_string(), oid(A))]
    );
}
