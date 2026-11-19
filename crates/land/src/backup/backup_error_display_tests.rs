//! Display for BackupError: each refusal names what it carries and says what moved. The Host, Git and Oplog arms
//! print oplog's or gitio's own text after their prefix.

use super::BackupError;
use gitio::{GitError, PushRefusal};
use oplog::{LockError, OpPhase, OpRecord, OpResult, OplogError};

fn git_error() -> GitError {
    GitError { args: vec!["read-tree".into(), "HEAD".into()], code: Some(128), stderr: "fatal: no tree\n".into() }
}

fn record(phase: OpPhase) -> OpRecord {
    OpRecord {
        op_id: "op-7".to_string(),
        key: "land-restore-backup:h1:op-7".to_string(),
        kind: "land-restore-backup".to_string(),
        actor: "42@host".to_string(),
        reason: "r".to_string(),
        at: 9,
        phase,
        moves: Vec::new(),
        result: Some(OpResult::Moved),
    }
}

fn refused(refusal: PushRefusal) -> String {
    BackupError::Refused { refname: "refs/mortar-backup/h1/heads/zone/a".to_string(), refusal }.to_string()
}

#[test]
fn bad_host_names_the_host_and_says_nothing_moved() {
    let text = BackupError::BadHost("a/b".to_string()).to_string();
    assert_eq!(
        text,
        "host name \"a/b\" is empty or holds '/', so it cannot name a backup namespace; nothing was pushed or \
         restored"
    );
}

#[test]
fn refused_names_the_backup_ref_and_says_what_moved() {
    let text = refused(PushRefusal::StaleLease);
    assert!(text.starts_with("the remote refused to move backup ref refs/mortar-backup/h1/heads/zone/a: "), "{text}");
    assert!(text.contains("it was not moved, other backup refs in the same push may have been"), "{text}");
}

#[test]
fn refused_says_why_for_each_refusal() {
    let cases = [
        (PushRefusal::StaleLease, "it moved on the remote since it was read (stale lease)"),
        (PushRefusal::NonFastForward, "the move is not a fast-forward"),
        (PushRefusal::Hook, "a hook on the remote declined it"),
        (PushRefusal::Other("[rejected] (fetch first)".to_string()), "git said [rejected] (fetch first)"),
    ];
    for (refusal, why) in cases {
        let text = refused(refusal);
        assert!(text.contains(&format!("zone/a: {why}; ")), "{text}");
    }
}

#[test]
fn op_failed_names_the_op_and_its_key_and_says_nothing_moved() {
    let text = BackupError::OpFailed(record(OpPhase::Failed)).to_string();
    assert!(text.contains("op op-7 (key land-restore-backup:h1:op-7) failed"), "{text}");
    assert!(text.contains("not at its expected value"), "{text}");
    assert!(text.contains("nothing was restored"), "{text}");
}

#[test]
fn op_repeated_names_the_key_and_says_nothing_moved() {
    let text = BackupError::OpRepeated(record(OpPhase::Done)).to_string();
    assert!(text.contains("op key land-restore-backup:h1:op-7 is already done"), "{text}");
    assert!(text.contains("this restore moved nothing"), "{text}");
}

#[test]
fn host_says_nothing_moved() {
    assert_eq!(BackupError::Host(LockError::HostUnknown("hostname failed".into())).to_string(), "backup host name: cannot read this host's name: hostname failed; check that `hostname` runs on this machine; nothing was pushed or restored");
}

#[test]
fn git_is_marked_as_backups() {
    assert_eq!(BackupError::Git(git_error()).to_string(), "backup git: git read-tree HEAD: exit code 128: fatal: no tree");
}

#[test]
fn oplog_is_marked_as_backups() {
    let shown = BackupError::Oplog(OplogError::UnknownOp { op_id: "op-404".into() }).to_string();
    assert!(shown == "backup op log: cannot restore: op op-404 is not in the op log; find the op id with mortar log"
        || shown == "backup op log: cannot restore: op op-404 is not in the op log; find the op id with mr log", "{shown}");
}
