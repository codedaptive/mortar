//! From<CommitFilesError>: the refusal comes back unchanged inside CommitFiles.

use crate::landing::{CommitFilesError, HookedCommitFilesError};
use gitio::Oid;

#[test]
fn a_commit_files_error_becomes_commit_files_with_the_same_refusal() {
    let landed = Oid::parse("1111111111111111111111111111111111111111").unwrap();
    let integration = Oid::parse("2222222222222222222222222222222222222222").unwrap();
    let error = HookedCommitFilesError::from(CommitFilesError::NotLanded { landed, integration });
    let HookedCommitFilesError::CommitFiles(CommitFilesError::NotLanded { landed, integration }) = error else {
        panic!("expected CommitFiles(NotLanded), got {error:?}");
    };
    assert_eq!(landed.as_str(), "1111111111111111111111111111111111111111");
    assert_eq!(integration.as_str(), "2222222222222222222222222222222222222222");
}
