//! remove_step_for: a zone folder is ZoneFolder whatever else; HEAD on another branch is OffBranch; a merge base
//! at the tip is Go; another merge base, or none, is NotJoined.

use gitio::Oid;

use super::remove_step_for::remove_step_for;
use super::RemoveStep;
use crate::record::FolderKind;

const TIP: &str = "0123456789abcdef0123456789abcdef01234567";
const OTHER: &str = "fedcba9876543210fedcba9876543210fedcba98";
const BRANCH: &str = "refs/heads/req/folders/rf5";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn a_contained_request_branch_is_go() {
    let step = remove_step_for(FolderKind::Request, BRANCH, BRANCH, &oid(TIP), Some(&oid(TIP)));
    assert_eq!(step, RemoveStep::Go);
}

#[test]
fn a_zone_folder_is_zone_folder_even_when_contained_and_on_branch() {
    let step = remove_step_for(FolderKind::Zone, BRANCH, BRANCH, &oid(TIP), Some(&oid(TIP)));
    assert_eq!(step, RemoveStep::ZoneFolder);
}

#[test]
fn a_zone_folder_is_zone_folder_off_its_branch() {
    let step = remove_step_for(FolderKind::Zone, "", "refs/heads/zone/folders", &oid(TIP), None);
    assert_eq!(step, RemoveStep::ZoneFolder);
}

#[test]
fn head_on_another_branch_is_off_branch() {
    let step = remove_step_for(FolderKind::Request, BRANCH, "refs/heads/main", &oid(TIP), Some(&oid(TIP)));
    assert_eq!(step, RemoveStep::OffBranch);
}

#[test]
fn a_merge_base_at_another_commit_is_not_joined() {
    let step = remove_step_for(FolderKind::Request, BRANCH, BRANCH, &oid(TIP), Some(&oid(OTHER)));
    assert_eq!(step, RemoveStep::NotJoined);
}

#[test]
fn no_merge_base_is_not_joined() {
    let step = remove_step_for(FolderKind::Request, BRANCH, BRANCH, &oid(TIP), None);
    assert_eq!(step, RemoveStep::NotJoined);
}
