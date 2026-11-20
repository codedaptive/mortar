use crate::{JoinError, JoinRunError, MarkedFile};
use gitio::{GitError, Oid};
use oplog::{OpPhase, OpRecord, OpResult, OplogError, RefMove};
use std::path::PathBuf;

const OLD: &str = "1111111111111111111111111111111111111111";
const NEW: &str = "2222222222222222222222222222222222222222";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn replay_carries_the_replay_refusal_and_says_join_moved_no_ref() {
    let text = JoinRunError::Replay(JoinError::TipMissing("refs/heads/topic".to_string())).to_string();
    assert_eq!(
        text,
        "join replay: tip ref refs/heads/topic does not exist (read by its full name); join moved no ref"
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
fn unrelated_names_both_refs_and_says_join_moved_no_ref() {
    let error = JoinRunError::Unrelated {
        zone_ref: "refs/heads/zone/join".to_string(),
        request_ref: "refs/heads/req/one".to_string(),
    };
    assert_eq!(
        error.to_string(),
        "request ref refs/heads/req/one shares no history with zone ref refs/heads/zone/join; join refuses and \
         moved no ref"
    );
}

#[test]
fn op_failed_names_the_op_and_its_key_and_says_join_moved_no_ref() {
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
    assert_eq!(
        JoinRunError::OpFailed(record).to_string(),
        "op op-7 (key join-rebase:refs/heads/req/one:k) failed: a ref was not at its expected value, so join moved \
         no ref; run the command again to read the refs afresh"
    );
}

#[test]
fn op_repeated_names_the_key_and_says_join_moved_no_ref() {
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
    assert_eq!(
        JoinRunError::OpRepeated(record).to_string(),
        "op key join-fast-forward:refs/heads/zone/join:op-8 is already done in the op log, so join moved no ref; run \
         the command again"
    );
}

#[test]
fn state_path_names_the_path_and_says_it_is_not_utf8() {
    assert_eq!(
        JoinRunError::StatePath(PathBuf::from("src/a.rs")).to_string(),
        "conflicted path src/a.rs is not UTF-8, so mortar-join.json cannot record it; join moved no ref"
    );
}

#[test]
fn state_write_names_the_path_and_the_io_error() {
    let error = JoinRunError::StateWrite {
        path: PathBuf::from("/repo/.git/mortar-join.json.tmp"),
        source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied here"),
    };
    assert_eq!(
        error.to_string(),
        "/repo/.git/mortar-join.json.tmp: denied here; mortar-join.json was not written and join moved no ref; check \
         that folder and its permissions, then run the command again"
    );
}

#[test]
fn nothing_to_resolve_names_the_state_path() {
    let error = JoinRunError::NothingToResolve { state: PathBuf::from("/repo/.git/mortar-join.json") };
    assert_eq!(
        error.to_string(),
        "nothing to resolve: /repo/.git/mortar-join.json does not exist, so no join is paused in this folder"
    );
}

#[test]
fn still_marked_names_each_file_and_its_lines() {
    let error = JoinRunError::StillMarked(vec![
        MarkedFile { path: PathBuf::from("src/a.rs"), lines: vec![3, 7, 11] },
        MarkedFile { path: PathBuf::from("b.txt"), lines: vec![1] },
    ]);
    assert_eq!(
        error.to_string(),
        "conflict markers remain; join made no commit and moved no ref:\n  src/a.rs (lines 3, 7, 11)\n  b.txt \
         (lines 1)\nremove them, then run `mr resolve` again"
    );
}

#[test]
fn state_stale_names_the_ref_both_tips_the_kept_file_and_the_way_out() {
    let error = JoinRunError::StateStale {
        request_ref: "refs/heads/request/r1".to_string(),
        expected: oid(OLD),
        actual: Some(oid(NEW)),
        state: PathBuf::from("/repo/.git/mortar-join.json"),
    };
    assert_eq!(
        error.to_string(),
        "request ref refs/heads/request/r1 is at 2222222222222222222222222222222222222222, not at \
         1111111111111111111111111111111111111111 where the paused join left it, so /repo/.git/mortar-join.json no \
         longer describes the request; the state file was kept and join moved no ref. To start over: remove \
         /repo/.git/mortar-join.json, then run `mr join` to replay the request afresh"
    );
}

#[test]
fn state_stale_with_the_ref_gone_says_it_does_not_exist() {
    let error = JoinRunError::StateStale {
        request_ref: "refs/heads/request/r1".to_string(),
        expected: oid(OLD),
        actual: None,
        state: PathBuf::from("/repo/.git/mortar-join.json"),
    };
    assert_eq!(
        error.to_string(),
        "request ref refs/heads/request/r1 does not exist, not at 1111111111111111111111111111111111111111 where the \
         paused join left it, so /repo/.git/mortar-join.json no longer describes the request; the state file was \
         kept and join moved no ref. To start over: remove /repo/.git/mortar-join.json, then run `mr join` to \
         replay the request afresh"
    );
}

#[test]
fn state_read_names_the_path_and_the_io_error() {
    let error = JoinRunError::StateRead {
        path: PathBuf::from("/repo/.git/mortar-join.json"),
        source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied here"),
    };
    assert_eq!(
        error.to_string(),
        "/repo/.git/mortar-join.json: denied here; the join state could not be read and join moved no ref"
    );
}

#[test]
fn state_corrupt_names_the_path_and_the_reason() {
    let error = JoinRunError::StateCorrupt {
        path: PathBuf::from("/repo/.git/mortar-join.json"),
        reason: "expected \"onto\" at byte 60".to_string(),
    };
    assert_eq!(
        error.to_string(),
        "/repo/.git/mortar-join.json is not a state join wrote: expected \"onto\" at byte 60; join moved no ref"
    );
}

#[test]
fn state_remove_names_the_path_and_says_the_ref_moved_and_to_resolve_again() {
    let error = JoinRunError::StateRemove {
        path: PathBuf::from("/repo/.git/mortar-join.json"),
        source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied here"),
    };
    assert_eq!(
        error.to_string(),
        "/repo/.git/mortar-join.json: denied here; the request ref was moved but the join state was not removed; \
         check that folder and its permissions, then run `mr resolve` again to finish"
    );
}

#[test]
fn file_read_names_the_path_and_the_io_error() {
    let error = JoinRunError::FileRead {
        path: PathBuf::from("/repo/src/a.rs"),
        source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied here"),
    };
    assert_eq!(
        error.to_string(),
        "/repo/src/a.rs: denied here; the conflicted file could not be checked for markers and join moved no ref"
    );
}

#[test]
fn conflicted_path_is_folder_names_the_path_and_the_way_out() {
    let text = JoinRunError::ConflictedPathIsFolder { path: PathBuf::from("src/gen") }.to_string();
    assert_eq!(
        text,
        "conflicted path src/gen names a folder; join moved no ref. A conflicted path must be a file or removed: \
         make it one, then run `mr resolve` again"
    );
}

#[test]
fn conflicted_path_refused_names_the_path_and_gits_reason() {
    let reason = "not usable paths (one is a folder holding the other): \"lib\" and \"lib/x\"".to_string();
    let text = JoinRunError::ConflictedPathRefused { path: Some(PathBuf::from("lib")), reason }.to_string();
    assert_eq!(
        text,
        "conflicted path lib was refused: not usable paths (one is a folder holding the other): \"lib\" and \
         \"lib/x\"; join moved no ref"
    );
}

#[test]
fn conflicted_path_refused_without_a_path_says_the_paths_were_refused() {
    let reason = "not ls-tree records for the paths asked (no final NUL): \"x\"".to_string();
    let text = JoinRunError::ConflictedPathRefused { path: None, reason }.to_string();
    assert_eq!(
        text,
        "the conflicted paths were refused: not ls-tree records for the paths asked (no final NUL): \"x\"; join \
         moved no ref"
    );
}
