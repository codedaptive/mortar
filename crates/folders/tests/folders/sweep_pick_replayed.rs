//! Zone integration tests of sweep_pick, with closed_snap_refs, stale_replayed_refs and extra_ref_verdict used for
//! real (S-79 as amended by #1264), with replay-record refs (S-121 (2)): one of a zone not in the zone list comes
//! after the closed snapshot refs and before the joined branches and the extras; a listed zone's is absent; one
//! listed twice is picked once; a refused extra is ExtraRefRefused with nothing picked, even with a stale one
//! present.

use folders::sweep::{sweep_pick, Leftover};
use folders::FolderError;

use crate::ref_pairs::{oid, picked, refs, strings as zones};

const A: &str = "0123456789abcdef0123456789abcdef01234567";
const B: &str = "fedcba9876543210fedcba9876543210fedcba98";
const C: &str = "1111111111111111111111111111111111111111";
const D: &str = "2222222222222222222222222222222222222222";
const E: &str = "3333333333333333333333333333333333333333";

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
