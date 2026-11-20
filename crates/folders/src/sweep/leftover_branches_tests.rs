//! leftover_branches: a req/<zone>/<request> branch checked out nowhere, with its zone branch present, is picked
//! with the zone's tip; a checked-out one, one with no zone branch, a try/ attempt branch, a three-segment name and a
//! non-req ref are not; input order is kept.

use super::leftover_branches::leftover_branches;
use super::Leftover;
use crate::test_ref_pairs::{oid, refs};

const A: &str = "0123456789abcdef0123456789abcdef01234567";
const B: &str = "fedcba9876543210fedcba9876543210fedcba98";
const C: &str = "1111111111111111111111111111111111111111";

#[test]
fn a_branch_with_no_worktree_and_a_zone_branch_is_picked() {
    let input = refs(&[(A, "refs/heads/req/z/r"), (B, "refs/heads/zone/z")]);
    assert_eq!(
        leftover_branches(&input, &[]),
        vec![Leftover {
            branch: "refs/heads/req/z/r".to_string(),
            tip: oid(A),
            zone_tip: oid(B),
        }]
    );
}

#[test]
fn a_checked_out_branch_is_skipped() {
    let input = refs(&[(A, "refs/heads/req/z/r"), (B, "refs/heads/zone/z")]);
    assert_eq!(leftover_branches(&input, &["refs/heads/req/z/r".to_string()]), Vec::<Leftover>::new());
}

#[test]
fn a_branch_whose_zone_branch_is_missing_is_skipped() {
    let input = refs(&[(A, "refs/heads/req/z/r"), (B, "refs/heads/zone/y")]);
    assert_eq!(leftover_branches(&input, &[]), Vec::<Leftover>::new());
}

#[test]
fn a_try_attempt_branch_is_not_picked() {
    let input = refs(&[(A, "refs/heads/try/z/r/1"), (B, "refs/heads/zone/z")]);
    assert_eq!(leftover_branches(&input, &[]), Vec::<Leftover>::new());
}

#[test]
fn a_three_segment_name_and_empty_segments_are_not_picked() {
    let input = refs(&[
        (A, "refs/heads/req/z/r/x"),
        (A, "refs/heads/req/z"),
        (A, "refs/heads/req//r"),
        (A, "refs/heads/req/z/"),
        (B, "refs/heads/zone/z"),
    ]);
    assert_eq!(leftover_branches(&input, &[]), Vec::<Leftover>::new());
}

#[test]
fn a_non_req_ref_is_not_picked() {
    let input = refs(&[
        (A, "refs/heads/main"),
        (A, "refs/mortar/snap/z/r"),
        (A, "refs/remotes/origin/req/z/r"),
        (B, "refs/heads/zone/z"),
    ]);
    assert_eq!(leftover_branches(&input, &[]), Vec::<Leftover>::new());
}

#[test]
fn input_order_is_kept() {
    let input = refs(&[
        (C, "refs/heads/req/y/b"),
        (A, "refs/heads/zone/z"),
        (B, "refs/heads/req/z/a"),
        (B, "refs/heads/zone/y"),
    ]);
    assert_eq!(
        leftover_branches(&input, &[]),
        vec![
            Leftover {
                branch: "refs/heads/req/y/b".to_string(),
                tip: oid(C),
                zone_tip: oid(B),
            },
            Leftover {
                branch: "refs/heads/req/z/a".to_string(),
                tip: oid(B),
                zone_tip: oid(A),
            },
        ]
    );
}
