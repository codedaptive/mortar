//! Display for BackupError: each test pins the whole Display text, in land's own words around the values the test
//! passes, except: the Host arm is land's prefix, oplog's LockError text, then land's suffix; the Oplog arm is land's
//! prefix then oplog's text; the Git arm is land's prefix then gitio's text.

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
fn bad_host_text_is_the_host_why_no_backup_namespace_then_nothing_pushed_and_nothing_restored() {
    let text = BackupError::BadHost("a/b".to_string()).to_string();
    assert_eq!(
        text,
        "host name \"a/b\" is empty or holds '/', so it cannot name a backup namespace; nothing was pushed or \
         restored"
    );
}

#[test]
fn refused_text_is_the_remote_refused_the_ref_each_refusals_why_what_moved_then_what_to_do() {
    let cases = [
        (
            PushRefusal::StaleLease,
            "the remote refused to move backup ref refs/mortar-backup/h1/heads/zone/a: it moved on the remote since \
             it was read (stale lease); it was not moved, other backup refs in the same push may have been; run \
             backup again to compare afresh",
        ),
        (
            PushRefusal::NonFastForward,
            "the remote refused to move backup ref refs/mortar-backup/h1/heads/zone/a: the move is not a \
             fast-forward; it was not moved, other backup refs in the same push may have been; run backup again to \
             compare afresh",
        ),
        (
            PushRefusal::Hook,
            "the remote refused to move backup ref refs/mortar-backup/h1/heads/zone/a: a hook on the remote \
             declined it; it was not moved, other backup refs in the same push may have been; run backup again to \
             compare afresh",
        ),
        (
            PushRefusal::Other("[rejected] (fetch first)".to_string()),
            "the remote refused to move backup ref refs/mortar-backup/h1/heads/zone/a: git said [rejected] (fetch \
             first); it was not moved, other backup refs in the same push may have been; run backup again to \
             compare afresh",
        ),
    ];
    for (refusal, expected) in cases {
        assert_eq!(refused(refusal), expected);
    }
}

#[test]
fn op_failed_text_is_the_op_its_key_failed_why_nothing_restored_then_what_to_do() {
    assert_eq!(
        BackupError::OpFailed(record(OpPhase::Failed)).to_string(),
        "op op-7 (key land-restore-backup:h1:op-7) failed: a local ref was not at its expected value, so nothing was \
         restored; run restore again to read the refs afresh"
    );
}

#[test]
fn op_repeated_text_is_the_key_already_done_in_the_op_log_nothing_moved_then_what_to_do() {
    assert_eq!(
        BackupError::OpRepeated(record(OpPhase::Done)).to_string(),
        "op key land-restore-backup:h1:op-7 is already done in the op log, so this restore moved nothing; run restore \
         again"
    );
}

#[test]
fn host_text_is_the_backup_prefix_cannot_read_the_host_name_the_reason_what_to_do_then_nothing_pushed_and_nothing_restored() {
    assert_eq!(BackupError::Host(LockError::HostUnknown("hostname failed".into())).to_string(), "backup host name: cannot read this host's name: hostname failed; check that `hostname` runs on this machine; nothing was pushed or restored");
}

#[test]
fn git_text_is_the_backup_prefix_then_gitios_command_exit_code_and_stderr() {
    assert_eq!(BackupError::Git(git_error()).to_string(), "backup git: git read-tree HEAD: exit code 128: fatal: no tree");
}

#[test]
fn oplog_text_is_the_backup_prefix_cannot_restore_the_op_not_in_the_log_then_what_to_do() {
    assert_eq!(BackupError::Oplog(OplogError::UnknownOp { op_id: "op-404".into() }).to_string(), "backup op log: cannot restore: op op-404 is not in the op log; find the op id with mr log");
}
