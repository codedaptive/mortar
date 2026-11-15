//! stale_replayed_refs: a replay-record ref whose zone is in the caller's zone list (exact match) is kept; one whose
//! zone is not is picked, every one when the list is empty; a zone may hold '/'; non-family names are left alone;
//! input order is kept.

use gitio::Oid;

use super::stale_replayed_refs::stale_replayed_refs;

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

fn picked(pairs: &[(&str, &str)]) -> Vec<(String, Oid)> {
    pairs.iter().map(|(name, hex)| (name.to_string(), oid(hex))).collect()
}

fn zones(list: &[&str]) -> Vec<String> {
    list.iter().map(|zone| zone.to_string()).collect()
}

#[test]
fn a_listed_zones_ref_is_kept() {
    let input = refs(&[(A, "refs/mortar/replayed/red/s")]);
    assert_eq!(stale_replayed_refs(&input, &zones(&["blue", "red"])), picked(&[]));
}

#[test]
fn an_unlisted_zones_ref_is_picked() {
    let input = refs(&[(A, "refs/mortar/replayed/blue/s")]);
    assert_eq!(stale_replayed_refs(&input, &zones(&["red"])), picked(&[("refs/mortar/replayed/blue/s", A)]));
}

#[test]
fn an_empty_list_picks_every_family_ref() {
    let input = refs(&[(A, "refs/mortar/replayed/red/s"), (B, "refs/mortar/replayed/team/land/t")]);
    assert_eq!(
        stale_replayed_refs(&input, &[]),
        picked(&[("refs/mortar/replayed/red/s", A), ("refs/mortar/replayed/team/land/t", B)])
    );
}

#[test]
fn a_zone_with_a_slash_is_kept_only_by_its_whole_name() {
    let land = refs(&[(A, "refs/mortar/replayed/team/land/x")]);
    assert_eq!(stale_replayed_refs(&land, &zones(&["team/land"])), picked(&[]));
    assert_eq!(stale_replayed_refs(&land, &zones(&["team"])), picked(&[("refs/mortar/replayed/team/land/x", A)]));
    let team = refs(&[(B, "refs/mortar/replayed/team/y")]);
    assert_eq!(stale_replayed_refs(&team, &zones(&["team/land"])), picked(&[("refs/mortar/replayed/team/y", B)]));
}

#[test]
fn the_zone_matches_exactly() {
    let input = refs(&[(A, "refs/mortar/replayed/red/s")]);
    assert_eq!(stale_replayed_refs(&input, &zones(&["Red"])), picked(&[("refs/mortar/replayed/red/s", A)]));
}

#[test]
fn non_family_names_left_alone() {
    let input = refs(&[
        (A, "refs/mortar/snap/z/r"),
        (A, "refs/heads/main"),
        (A, "refs/heads/integration"),
        (A, "refs/heads/zone/z"),
        (A, "refs/mortar/replayed/onlyone"),
        (A, "refs/mortar/replayed//x"),
        (A, "refs/mortar/replayedx/z/s"),
        (A, "refs/mortar/replayed/z/"),
        (A, "refs/mortar/replayed/a//b/s"),
    ]);
    assert_eq!(stale_replayed_refs(&input, &[]), picked(&[]));
}

#[test]
fn input_order_kept() {
    let input = refs(&[
        (D, "refs/mortar/replayed/z/d"),
        (A, "refs/mortar/replayed/live/c"),
        (B, "refs/mortar/replayed/a/b/s"),
        (C, "refs/heads/main"),
        (A, "refs/mortar/replayed/y/t"),
    ]);
    assert_eq!(
        stale_replayed_refs(&input, &zones(&["live"])),
        picked(&[
            ("refs/mortar/replayed/z/d", D),
            ("refs/mortar/replayed/a/b/s", B),
            ("refs/mortar/replayed/y/t", A),
        ])
    );
}
