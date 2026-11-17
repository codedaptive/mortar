//! sweep_pick: a joined branch comes with its snapshot ref, or alone when it has none; closed snapshot refs, then
//! joined branches, then accepted extras, in that order; a duplicate refname is dropped; a refused extra is
//! ExtraRefRefused with nothing picked; a skipped extra is absent. With replay-record refs (S-121 (2)): one of a
//! zone not in the zone list comes after the closed snapshot refs and before the joined branches and the extras; a
//! listed zone's is absent; one listed twice is picked once; a refused extra is ExtraRefRefused with nothing
//! picked, even with a stale one present.

use oplog::ProtectedRef;

use super::{sweep_pick, Leftover};
use crate::test_ref_pairs::{oid, picked, refs, strings as names, strings as zones};
use crate::FolderError;

const A: &str = "0123456789abcdef0123456789abcdef01234567";
const B: &str = "fedcba9876543210fedcba9876543210fedcba98";
const C: &str = "1111111111111111111111111111111111111111";
const D: &str = "2222222222222222222222222222222222222222";
const E: &str = "3333333333333333333333333333333333333333";

fn leftover(branch: &str, tip: &str, zone_tip: &str) -> Leftover {
    Leftover {
        branch: branch.to_string(),
        tip: oid(tip),
        zone_tip: oid(zone_tip),
    }
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

#[test]
fn a_stale_replayed_ref_comes_after_closed_snapshots_and_before_joined_and_extras() {
    let input = refs(&[
        (D, "refs/t3/checkpoint/1"),
        (E, "refs/mortar/replayed/gone/aaaa"),
        (A, "refs/heads/req/z/r"),
        (B, "refs/heads/zone/z"),
        (C, "refs/mortar/snap/z/r"),
        (D, "refs/mortar/snap/y/closed"),
    ]);
    let joined = [Leftover {
        branch: "refs/heads/req/z/r".to_string(),
        tip: oid(A),
        zone_tip: oid(B),
    }];
    let extras = ["refs/t3/checkpoint/1".to_string()];
    assert_eq!(
        sweep_pick(&input, &zones(&["z", "y"]), &joined, &extras, &[]).unwrap(),
        picked(&[
            ("refs/mortar/snap/y/closed", D),
            ("refs/mortar/replayed/gone/aaaa", E),
            ("refs/heads/req/z/r", A),
            ("refs/mortar/snap/z/r", C),
            ("refs/t3/checkpoint/1", D),
        ])
    );
}

#[test]
fn a_listed_zones_replayed_ref_is_absent() {
    let input = refs(&[(C, "refs/mortar/replayed/z/aaaa"), (D, "refs/mortar/replayed/gone/bbbb")]);
    assert_eq!(
        sweep_pick(&input, &zones(&["z"]), &[], &[], &[]).unwrap(),
        picked(&[("refs/mortar/replayed/gone/bbbb", D)])
    );
}

#[test]
fn a_replayed_ref_listed_twice_is_picked_once() {
    let input = refs(&[(C, "refs/mortar/replayed/z/aaaa"), (C, "refs/mortar/replayed/z/aaaa")]);
    assert_eq!(
        sweep_pick(&input, &[], &[], &[], &[]).unwrap(),
        picked(&[("refs/mortar/replayed/z/aaaa", C)])
    );
}

#[test]
fn a_refused_extra_is_the_error_even_with_a_stale_replayed_ref() {
    let input = refs(&[(C, "refs/mortar/replayed/z/aaaa"), (D, "refs/t3/checkpoint/1")]);
    let extras = ["refs/t3/checkpoint/1".to_string(), "refs/mortar/replayed/z/aaaa".to_string()];
    match sweep_pick(&input, &[], &[], &extras, &[]) {
        Err(FolderError::ExtraRefRefused { refname, reason }) => {
            assert_eq!(
                (refname.as_str(), reason.as_str()),
                ("refs/mortar/replayed/z/aaaa", "is under refs/mortar/")
            );
        }
        other => panic!("expected ExtraRefRefused, got {other:?}"),
    }
}
