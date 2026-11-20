//! zone_start_for: the tip wins, else the start creates the branch, else the zone branch is missing.

use gitio::Oid;

use super::zone_start_for::zone_start_for;
use super::{FolderError, ZoneStart};

const TIP: &str = "489d82745705a031306253da29bdad7aee19d2e5";
const START: &str = "1111111111111111111111111111111111111111";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn a_tip_alone_is_existing() {
    let got = zone_start_for(Some(&oid(TIP)), None, "zone/folders").unwrap();
    assert_eq!(got, ZoneStart::Existing(oid(TIP)));
}

#[test]
fn a_tip_wins_over_a_different_start() {
    let got = zone_start_for(Some(&oid(TIP)), Some(&oid(START)), "zone/folders").unwrap();
    assert_eq!(got, ZoneStart::Existing(oid(TIP)));
}

#[test]
fn no_tip_and_a_start_is_create() {
    let got = zone_start_for(None, Some(&oid(START)), "zone/folders").unwrap();
    assert_eq!(got, ZoneStart::Create(oid(START)));
}

#[test]
fn neither_is_zone_branch_missing() {
    let got = zone_start_for(None, None, "zone/folders");
    assert!(matches!(got, Err(FolderError::ZoneBranchMissing { branch }) if branch == "zone/folders"));
}
