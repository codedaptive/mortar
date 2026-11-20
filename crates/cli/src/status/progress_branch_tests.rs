//! progress_branch: a request folder reads the request branch, attempt k its own try branch; the plan's refusals
//! come through.

use folders::FolderKind;

use super::progress_branch;

#[test]
fn a_request_folder_reads_the_request_branch() {
    assert_eq!(progress_branch("core", "task", FolderKind::Request).unwrap(), "req/core/task");
}

#[test]
fn a_zone_folder_reads_the_request_branch() {
    assert_eq!(progress_branch("core", "task", FolderKind::Zone).unwrap(), "req/core/task");
}

#[test]
fn attempt_two_reads_its_own_try_branch() {
    assert_eq!(progress_branch("core", "task", FolderKind::Attempt(2)).unwrap(), "try/core/task/2");
}

#[test]
fn attempt_zero_is_refused() {
    assert!(progress_branch("core", "task", FolderKind::Attempt(0)).is_err());
}

#[test]
fn a_bad_name_is_refused_for_every_kind() {
    for kind in [FolderKind::Request, FolderKind::Attempt(1)] {
        assert!(progress_branch("core", "../task", kind).is_err(), "{kind:?}");
    }
}
