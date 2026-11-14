//! closed_snap_refs: a closed request's snapshot ref is picked; a request branch or an attempt branch keeps it
//! open, by exact name; odd-shaped snapshot refs and other refs are ignored; input order is kept.

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
    let input = refs(&[(B, "refs/mortar/snap/z/r"), (A, "refs/heads/req/z/r/try-1")]);
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
