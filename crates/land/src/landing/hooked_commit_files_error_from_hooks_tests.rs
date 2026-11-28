//! From<CommitPathsHooksError>: Paths comes back inside CommitFiles(Paths), HookRefused as Refused, each unchanged.

use crate::landing::{CommitFilesError, HookedCommitFilesError};
use gitio::HookRefusal;
use snapshot::{CommitPathsError, CommitPathsHooksError};
use std::path::PathBuf;

#[test]
fn a_hooks_error_becomes_commit_files_paths_or_refused() {
    let paths = HookedCommitFilesError::from(CommitPathsHooksError::Paths(CommitPathsError::Absolute {
        path: PathBuf::from("/etc/passwd"),
    }));
    let HookedCommitFilesError::CommitFiles(CommitFilesError::Paths(CommitPathsError::Absolute { path })) = paths
    else {
        panic!("expected CommitFiles(Paths(Absolute)), got {paths:?}");
    };
    assert_eq!(path, PathBuf::from("/etc/passwd"), "Paths row");

    let refused = HookedCommitFilesError::from(CommitPathsHooksError::HookRefused(HookRefusal {
        hook: "commit-msg",
        code: Some(1),
        stdout: b"out\n".to_vec(),
        stderr: b"bad message\n".to_vec(),
    }));
    let HookedCommitFilesError::Refused(refusal) = refused else {
        panic!("expected Refused, got {refused:?}");
    };
    let expected =
        HookRefusal { hook: "commit-msg", code: Some(1), stdout: b"out\n".to_vec(), stderr: b"bad message\n".to_vec() };
    assert_eq!(refusal, expected, "HookRefused row");
}
