//! Actual cross-kit error objects/Display implementations. Expected text is literal, never the inner formatter's output.
use gitio::GitError;
use land::{CulpritError, RereviewError, SquashError};
use oplog::{ChangeIdError, MortarTrailerError, OplogError};
use testkit::TestRepo;

fn git_error() -> GitError {
    GitError { args: vec!["read-tree".into(), "HEAD".into()], code: Some(128), stderr: "fatal: no tree\n".into() }
}
fn bad_id() -> ChangeIdError {
    ChangeIdError::Malformed { text: "bad".into(), reason: "expected 32 characters".into() }
}

#[test]
fn rit_squash_git_display_is_gitios_command_exit_and_trimmed_stderr_without_an_added_prefix() {
    let _repo = TestRepo::new().unwrap();
    assert_eq!(SquashError::Git(git_error()).to_string(), "git read-tree HEAD: exit code 128: fatal: no tree");
}

#[test]
fn rit_squash_oplog_display_prefix_keeps_oplogs_complete_refusal_and_next_step() {
    let _repo = TestRepo::new().unwrap();
    let inner = OplogError::Malformed { line: Some(7), reason: "missing op_id".into() };
    assert_eq!(SquashError::Oplog(inner).to_string(),
        "squash op log: op log line 7 is not a record (missing op_id); mortar did not write it, so check what did, then repair or remove that line of mortar/oplog.jsonl");
}

#[test]
fn rit_squash_trailers_display_preserves_both_oplog_and_gitio_error_layers() {
    let _repo = TestRepo::new().unwrap();
    let inner = MortarTrailerError::Git(git_error());
    assert_eq!(SquashError::Trailers(inner).to_string(), "squash trailers: git trailers failed: git read-tree HEAD: exit code 128: fatal: no tree");
}

#[test]
fn rit_squash_change_id_display_keeps_oplogs_bad_text_and_reason_after_the_land_prefix() {
    let _repo = TestRepo::new().unwrap();
    assert_eq!(SquashError::ChangeId(bad_id()).to_string(), "squash change ID: not a change ID: \"bad\": expected 32 characters");
}

#[test]
fn rit_culprit_trailers_display_is_the_inner_oplog_change_id_text_without_a_land_prefix() {
    let _repo = TestRepo::new().unwrap();
    assert_eq!(CulpritError::Trailers(MortarTrailerError::ChangeId(bad_id())).to_string(),
        "not a change ID: \"bad\": expected 32 characters");
}

#[test]
fn rit_rereview_git_display_adds_its_prefix_to_the_real_gitio_error_text() {
    let _repo = TestRepo::new().unwrap();
    assert_eq!(RereviewError::Git(git_error()).to_string(), "git failed: git read-tree HEAD: exit code 128: fatal: no tree");
}

#[test]
fn rit_rereview_trailers_display_is_the_real_nested_oplog_gitio_text_without_a_land_prefix() {
    let _repo = TestRepo::new().unwrap();
    assert_eq!(RereviewError::Trailers(MortarTrailerError::Git(git_error())).to_string(),
        "git trailers failed: git read-tree HEAD: exit code 128: fatal: no tree");
}
