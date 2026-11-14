//! step_refusal: Go refuses nothing; each other step is its refusal, naming the folder and the step's names.

use std::path::{Path, PathBuf};

use super::step_refusal::step_refusal;
use super::RemoveStep;
use crate::make::FolderError;

fn refusal(step: RemoveStep) -> Option<FolderError> {
    step_refusal(
        step,
        Path::new("/wt/req-a"),
        "folders",
        "refs/heads/req/folders/WO-1",
        "refs/heads/req/folders/WO-2",
        "refs/heads/zone/folders",
    )
}

#[test]
fn go_refuses_nothing() {
    assert!(refusal(RemoveStep::Go).is_none());
}

#[test]
fn zone_folder_names_the_folder_and_zone() {
    match refusal(RemoveStep::ZoneFolder) {
        Some(FolderError::ZoneFolder { path, zone }) => {
            assert_eq!(path, PathBuf::from("/wt/req-a"));
            assert_eq!(zone, "folders");
        }
        other => panic!("expected ZoneFolder, got {other:?}"),
    }
}

#[test]
fn off_branch_is_not_a_folder_naming_head_and_expected() {
    match refusal(RemoveStep::OffBranch) {
        Some(FolderError::NotAFolder { path, reason }) => {
            assert_eq!(path, PathBuf::from("/wt/req-a"));
            assert_eq!(
                reason,
                "its HEAD is on refs/heads/req/folders/WO-1, not its request branch refs/heads/req/folders/WO-2"
            );
        }
        other => panic!("expected NotAFolder, got {other:?}"),
    }
}

#[test]
fn not_joined_names_the_branch_and_zone_branch() {
    match refusal(RemoveStep::NotJoined) {
        Some(FolderError::NotJoined {
            path,
            branch,
            zone_branch,
        }) => {
            assert_eq!(path, PathBuf::from("/wt/req-a"));
            assert_eq!(branch, "refs/heads/req/folders/WO-1");
            assert_eq!(zone_branch, "refs/heads/zone/folders");
        }
        other => panic!("expected NotJoined, got {other:?}"),
    }
}
