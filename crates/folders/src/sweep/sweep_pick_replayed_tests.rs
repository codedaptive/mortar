//! sweep_pick with replay-record refs (S-121 (2)): one of a zone not in the zone list comes after the closed
//! snapshot refs and before the joined branches and the extras; a listed zone's is absent; one listed twice is
//! picked once; a refused extra is ExtraRefRefused with nothing picked, even with a stale one present.

use gitio::Oid;

use super::sweep_pick::sweep_pick;
use super::Leftover;
use crate::make::FolderError;

const A: &str = "0123456789abcdef0123456789abcdef01234567";
const B: &str = "fedcba9876543210fedcba9876543210fedcba98";
const C: &str = "1111111111111111111111111111111111111111";
const D: &str = "2222222222222222222222222222222222222222";
const E: &str = "3333333333333333333333333333333333333333";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn refs(names: &[(&str, &str)]) -> Vec<(Oid, String)> {
    names.iter().map(|(hex, name)| (oid(hex), name.to_string())).collect()
}

fn picked(pairs: &[(&str, &str)]) -> Vec<(String, Oid)> {
    pairs.iter().map(|(name, hex)| (name.to_string(), oid(hex))).collect()
}

fn zones(list: &[&str]) -> Vec<String> {
    list.iter().map(|zone| zone.to_string()).collect()
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
