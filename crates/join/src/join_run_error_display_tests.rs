use crate::{JoinError, JoinRunError};
use gitio::{GitError, Oid};
use oplog::{OpPhase, OpRecord, OpResult, OplogError, RefMove};
use std::path::PathBuf;

const OLD: &str = "1111111111111111111111111111111111111111";
const NEW: &str = "2222222222222222222222222222222222222222";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn replay_carries_the_replay_refusal_and_says_nothing_moved() {
    let text = JoinRunError::Replay(JoinError::TipMissing("refs/heads/topic".to_string())).to_string();
    assert_eq!(
        text,
        "join replay: tip ref refs/heads/topic does not exist (read by its full name); no ref was moved"
    );
}

#[test]
fn oplog_names_the_op_log_and_the_refused_ref() {
    let error = OplogError::BadMove { refname: "refs/heads/x".to_string(), reason: "is wrong".to_string() };
    let text = JoinRunError::Oplog(error).to_string();
    assert!(text.starts_with("join op log: "), "{text}");
    assert!(text.contains("\"refs/heads/x\""), "{text}");
    assert!(text.contains("is wrong"), "{text}");
}

#[test]
fn git_shows_args_code_and_stderr() {
    let error = GitError {
        args: vec!["merge-base".to_string(), OLD.to_string()],
        code: Some(128),
        stderr: "fatal: bad object\n".to_string(),
    };
    assert_eq!(
        JoinRunError::Git(error).to_string(),
        "git merge-base 1111111111111111111111111111111111111111: exit code 128: fatal: bad object"
    );
}

#[test]
fn zone_missing_names_the_zone_ref_and_that_join_does_not_create_it() {
    let text = JoinRunError::ZoneMissing("refs/heads/zone/join".to_string()).to_string();
    assert!(text.contains("zone ref refs/heads/zone/join does not exist"), "{text}");
    assert!(text.contains("join does not create it"), "{text}");
}

#[test]
fn request_missing_names_the_request_ref() {
    let text = JoinRunError::RequestMissing("refs/heads/req/one".to_string()).to_string();
    assert!(text.contains("request ref refs/heads/req/one does not exist"), "{text}");
}

#[test]
fn unrelated_names_both_refs_and_says_nothing_moved() {
    let error = JoinRunError::Unrelated {
        zone_ref: "refs/heads/zone/join".to_string(),
        request_ref: "refs/heads/req/one".to_string(),
    };
    let text = error.to_string();
    assert!(text.contains("refs/heads/req/one shares no history with zone ref refs/heads/zone/join"), "{text}");
    assert!(text.contains("no ref was moved"), "{text}");
}

#[test]
fn op_failed_names_the_op_and_its_key_and_says_nothing_moved() {
    let record = OpRecord {
        op_id: "op-7".to_string(),
        key: "join-rebase:refs/heads/req/one:k".to_string(),
        kind: "join-rebase".to_string(),
        actor: "42@host".to_string(),
        reason: "r".to_string(),
        at: 9,
        phase: OpPhase::Failed,
        moves: vec![RefMove { refname: "refs/heads/req/one".to_string(), old: Some(oid(OLD)), new: oid(NEW) }],
        result: Some(OpResult::Stale { refname: "refs/heads/req/one".to_string(), actual: None }),
    };
    let text = JoinRunError::OpFailed(record).to_string();
    assert!(text.contains("op op-7 (key join-rebase:refs/heads/req/one:k) failed"), "{text}");
    assert!(text.contains("not at its expected value"), "{text}");
    assert!(text.contains("nothing was moved"), "{text}");
}

#[test]
fn op_repeated_names_the_key_and_says_nothing_moved() {
    let record = OpRecord {
        op_id: "op-8".to_string(),
        key: "join-fast-forward:refs/heads/zone/join:op-8".to_string(),
        kind: "join-fast-forward".to_string(),
        actor: "42@host".to_string(),
        reason: "r".to_string(),
        at: 9,
        phase: OpPhase::Done,
        moves: vec![RefMove { refname: "refs/heads/zone/join".to_string(), old: Some(oid(OLD)), new: oid(NEW) }],
        result: Some(OpResult::Moved),
    };
    let text = JoinRunError::OpRepeated(record).to_string();
    assert!(text.contains("op key join-fast-forward:refs/heads/zone/join:op-8 is already done"), "{text}");
    assert!(text.contains("this join moved nothing"), "{text}");
}

#[test]
fn state_path_names_the_path_and_says_it_is_not_utf8() {
    let text = JoinRunError::StatePath(PathBuf::from("src/a.rs")).to_string();
    assert!(text.contains("conflicted path src/a.rs is not UTF-8"), "{text}");
    assert!(text.contains("mortar-join.json"), "{text}");
}

#[test]
fn state_write_names_the_path_and_the_io_error() {
    let error = JoinRunError::StateWrite {
        path: PathBuf::from("/repo/.git/mortar-join.json.tmp"),
        source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied here"),
    };
    let text = error.to_string();
    assert!(text.starts_with("/repo/.git/mortar-join.json.tmp: denied here"), "{text}");
    assert!(text.contains("mortar-join.json was not written"), "{text}");
}
