//! sweep_pick: a joined branch comes with its snapshot ref, or alone when it has none; closed
//! snapshot refs, then joined branches, then accepted extras, in that order; a duplicate refname is dropped; a
//! refused extra is ExtraRefRefused with nothing picked; a skipped extra is absent.

use gitio::Oid;
use oplog::ProtectedRef;

use super::sweep_pick::sweep_pick;
use super::Leftover;
use crate::make::FolderError;

const A: &str = "0123456789abcdef0123456789abcdef01234567";
const B: &str = "fedcba9876543210fedcba9876543210fedcba98";
const C: &str = "1111111111111111111111111111111111111111";
const D: &str = "2222222222222222222222222222222222222222";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn refs(names: &[(&str, &str)]) -> Vec<(Oid, String)> {
    names.iter().map(|(hex, name)| (oid(hex), name.to_string())).collect()
}

fn leftover(branch: &str, tip: &str, zone_tip: &str) -> Leftover {
    Leftover {
        branch: branch.to_string(),
        tip: oid(tip),
        zone_tip: oid(zone_tip),
    }
}

fn picked(pairs: &[(&str, &str)]) -> Vec<(String, Oid)> {
    pairs.iter().map(|(name, hex)| (name.to_string(), oid(hex))).collect()
}

fn names(list: &[&str]) -> Vec<String> {
    list.iter().map(|name| name.to_string()).collect()
}

#[test]
fn a_joined_branch_comes_with_its_snapshot_ref() {
    let input = refs(&[(A, "refs/heads/req/z/r"), (B, "refs/heads/zone/z"), (C, "refs/mortar/snap/z/r")]);
    let joined = [leftover("refs/heads/req/z/r", A, B)];
    assert_eq!(
        sweep_pick(&input, &[], &joined, &[], &[]).unwrap(),
        picked(&[("refs/heads/req/z/r", A), ("refs/mortar/snap/z/r", C)])
    );
}

#[test]
fn a_joined_branch_whose_name_plan_would_refuse_still_comes_with_its_snapshot_ref() {
    let input = refs(&[(A, "refs/heads/req/z/r~1"), (B, "refs/heads/zone/z"), (C, "refs/mortar/snap/z/r~1")]);
    let joined = [leftover("refs/heads/req/z/r~1", A, B)];
    assert_eq!(
        sweep_pick(&input, &[], &joined, &[], &[]).unwrap(),
        picked(&[("refs/heads/req/z/r~1", A), ("refs/mortar/snap/z/r~1", C)])
    );
}

#[test]
fn a_joined_branch_with_no_snapshot_ref_comes_alone() {
    let input = refs(&[(A, "refs/heads/req/z/r"), (B, "refs/heads/zone/z"), (C, "refs/mortar/snap/z/other")]);
    let joined = [leftover("refs/heads/req/z/r", A, B)];
    assert_eq!(
        sweep_pick(&input, &[], &joined, &[], &[]).unwrap(),
        picked(&[("refs/mortar/snap/z/other", C), ("refs/heads/req/z/r", A)])
    );
}

#[test]
fn closed_snapshot_refs_then_joined_then_extras() {
    let input = refs(&[
        (D, "refs/t3/checkpoint/1"),
        (A, "refs/heads/req/z/r"),
        (B, "refs/heads/zone/z"),
        (C, "refs/mortar/snap/z/r"),
        (D, "refs/mortar/snap/y/closed"),
    ]);
    let joined = [leftover("refs/heads/req/z/r", A, B)];
    assert_eq!(
        sweep_pick(&input, &[], &joined, &names(&["refs/t3/checkpoint/1"]), &[]).unwrap(),
        picked(&[
            ("refs/mortar/snap/y/closed", D),
            ("refs/heads/req/z/r", A),
            ("refs/mortar/snap/z/r", C),
            ("refs/t3/checkpoint/1", D),
        ])
    );
}

#[test]
fn a_duplicate_is_dropped_keeping_the_first() {
    let input = refs(&[(A, "refs/t3/checkpoint/1"), (B, "refs/mortar/snap/y/closed")]);
    let extras = names(&["refs/t3/checkpoint/1", "refs/t3/checkpoint/1"]);
    assert_eq!(
        sweep_pick(&input, &[], &[], &extras, &[]).unwrap(),
        picked(&[("refs/mortar/snap/y/closed", B), ("refs/t3/checkpoint/1", A)])
    );
}

#[test]
fn a_joined_branch_listed_twice_is_picked_once() {
    let input = refs(&[(A, "refs/heads/req/z/r"), (B, "refs/heads/zone/z"), (C, "refs/mortar/snap/z/r")]);
    let joined = [leftover("refs/heads/req/z/r", A, B), leftover("refs/heads/req/z/r", A, B)];
    assert_eq!(
        sweep_pick(&input, &[], &joined, &[], &[]).unwrap(),
        picked(&[("refs/heads/req/z/r", A), ("refs/mortar/snap/z/r", C)])
    );
}

#[test]
fn a_refused_extra_is_the_error_even_with_refs_to_sweep() {
    let input = refs(&[
        (A, "refs/heads/req/z/r"),
        (B, "refs/heads/zone/z"),
        (C, "refs/mortar/snap/y/closed"),
        (D, "refs/t3/checkpoint/1"),
    ]);
    let joined = [leftover("refs/heads/req/z/r", A, B)];
    let protected = [ProtectedRef {
        refname: "refs/proven/z".to_string(),
        by: "land".to_string(),
    }];
    let extras = names(&["refs/t3/checkpoint/1", "refs/proven/z", "refs/heads/main"]);
    match sweep_pick(&input, &[], &joined, &extras, &protected) {
        Err(FolderError::ExtraRefRefused { refname, reason }) => {
            assert_eq!((refname.as_str(), reason.as_str()), ("refs/proven/z", "is protected by land"));
        }
        other => panic!("expected ExtraRefRefused, got {other:?}"),
    }
}

#[test]
fn a_skipped_extra_is_absent() {
    let input = refs(&[(A, "refs/t3/checkpoint/1")]);
    let extras = names(&["refs/t3/checkpoint/gone", "refs/t3/checkpoint/1"]);
    assert_eq!(sweep_pick(&input, &[], &[], &extras, &[]).unwrap(), picked(&[("refs/t3/checkpoint/1", A)]));
}
