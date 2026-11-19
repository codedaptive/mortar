//! Display for SquashError's refusals: each names what it carries and says what moved.
//! The Git, Oplog, Trailers and ChangeId texts embed gitio's or oplog's own failure text.

use super::SquashError;
use gitio::{GitError, Oid};
use oplog::{ChangeIdError, MortarTrailerError, OpPhase, OpRecord, OpResult, OplogError};

fn a() -> Oid { Oid::parse("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap() }
fn b() -> Oid { Oid::parse("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap() }

fn record(phase: OpPhase) -> OpRecord {
    OpRecord {
        op_id: "op-7".to_string(),
        key: "land-squash:refs/mortar/squash/land:none:k:op-7".to_string(),
        kind: "land-squash".to_string(),
        actor: "42@host".to_string(),
        reason: "r".to_string(),
        at: 9,
        phase,
        moves: Vec::new(),
        result: Some(OpResult::Moved),
    }
}

#[test]
fn ref_missing_names_the_ref_and_says_nothing_moved() {
    let text = SquashError::RefMissing { refname: "refs/heads/zone/land".to_string() }.to_string();
    assert_eq!(text, "ref refs/heads/zone/land does not exist; no commit was written and no ref was moved");
}

#[test]
fn op_failed_names_the_op_and_its_key_and_says_nothing_moved() {
    let text = SquashError::OpFailed(record(OpPhase::Failed)).to_string();
    assert!(text.contains("op op-7 (key land-squash:refs/mortar/squash/land:none:k:op-7) failed"), "{text}");
    assert!(text.contains("not at its expected value"), "{text}");
    assert!(text.contains("nothing was moved"), "{text}");
}

#[test]
fn op_repeated_names_the_key_and_says_nothing_moved() {
    let text = SquashError::OpRepeated(record(OpPhase::Done)).to_string();
    assert!(text.contains("op key land-squash:refs/mortar/squash/land:none:k:op-7 is already done"), "{text}");
    assert!(text.contains("this squash moved nothing"), "{text}");
}

#[test]
fn attr_source_missing_names_the_commit_and_says_nothing_moved() {
    assert_eq!(SquashError::AttrSourceMissing { commit: a() }.to_string(), "attribute source commit aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa does not exist, so the merge cannot read its attributes; no commit was written and no ref was moved");
}

#[test]
fn unrelated_names_both_tips_and_says_nothing_moved() {
    assert_eq!(SquashError::Unrelated { origin: a(), source: b() }.to_string(), "zone tip bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb shares no history with origin aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa; nothing was squashed and no ref was moved");
}

fn git_error() -> GitError {
    GitError { args: vec!["read-tree".into(), "HEAD".into()], code: Some(128), stderr: "fatal: no tree\n".into() }
}
fn bad_id() -> ChangeIdError {
    ChangeIdError::Malformed { text: "bad".into(), reason: "expected 32 characters".into() }
}

#[test]
fn squash_git_display_is_gitios_command_exit_and_trimmed_stderr_without_an_added_prefix() {
    assert_eq!(SquashError::Git(git_error()).to_string(), "git read-tree HEAD: exit code 128: fatal: no tree");
}

#[test]
fn squash_oplog_display_prefix_keeps_oplogs_complete_refusal_and_next_step() {
    let inner = OplogError::Malformed { line: Some(7), reason: "missing op_id".into() };
    assert_eq!(SquashError::Oplog(inner).to_string(),
        "squash op log: op log line 7 is not a record (missing op_id); mortar did not write it, so check what did, then repair or remove that line of mortar/oplog.jsonl");
}

#[test]
fn squash_trailers_display_preserves_both_oplog_and_gitio_error_layers() {
    let inner = MortarTrailerError::Git(git_error());
    assert_eq!(SquashError::Trailers(inner).to_string(), "squash trailers: git trailers failed: git read-tree HEAD: exit code 128: fatal: no tree");
}

#[test]
fn squash_change_id_display_keeps_oplogs_bad_text_and_reason_after_the_squash_prefix() {
    assert_eq!(SquashError::ChangeId(bad_id()).to_string(), "squash change ID: not a change ID: \"bad\": expected 32 characters");
}
