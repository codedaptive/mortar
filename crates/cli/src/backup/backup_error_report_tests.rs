//! backup_error_report: every BackupError arm for both verbs, each PushRefusal arm with its own reason and next
//! step, in the cli's own words; never gitio's or oplog's texts.

use gitio::{GitError, PushRefusal};
use land::BackupError;
use oplog::{LockError, OpPhase, OpRecord, OpResult, OplogError};

use super::backup_error_report;
use crate::report::{ExitCode, Report};

const BACKUP_REF: &str = "refs/mortar-backup/build-mac-02/heads/zone/cli";

fn record(phase: OpPhase) -> OpRecord {
    OpRecord {
        op_id: "op-7".to_string(),
        key: "land-restore-backup:build-mac-02:op-7".to_string(),
        kind: "land-restore-backup".to_string(),
        actor: "42@build-mac-02".to_string(),
        reason: "r".to_string(),
        at: 9,
        phase,
        moves: Vec::new(),
        result: Some(OpResult::Moved),
    }
}

fn refused(verb: &str, line: &str, owner: &str, next: &str) -> Report {
    Report {
        verb: verb.to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![line.to_string()],
        owner: Some(owner.to_string()),
        next: next.to_string(),
        fields: Vec::new(),
    }
}

fn failed(verb: &str, line: &str) -> Report {
    Report {
        verb: verb.to_string(),
        exit: ExitCode::Failure,
        outcome: "failed".to_string(),
        lines: vec![line.to_string(), "nothing half-done is left".to_string()],
        owner: None,
        next: "repeat the same command".to_string(),
        fields: Vec::new(),
    }
}

fn push_refused(refusal: PushRefusal) -> Report {
    backup_error_report("backup", &BackupError::Refused { refname: BACKUP_REF.to_string(), refusal }, "origin")
}

#[test]
fn an_unreadable_host_name_is_a_failure_in_the_clis_words_for_both_verbs() {
    for verb in ["backup", "restore-backup"] {
        let error = BackupError::Host(LockError::HostUnknown("oplog host text".to_string()));
        assert_eq!(
            backup_error_report(verb, &error, "origin"),
            failed(verb, "this machine's host name could not be read")
        );
    }
}

#[test]
fn a_bad_host_on_backup_is_this_machines_host_name() {
    let error = BackupError::BadHost("a/b".to_string());
    assert_eq!(
        backup_error_report("backup", &error, "origin"),
        refused(
            "backup",
            "host \"a/b\" cannot name a backup namespace: it is empty or holds '/'",
            "this machine's host name",
            "give this machine a host name that holds no '/', then repeat the same command",
        )
    );
}

#[test]
fn a_bad_host_on_restore_backup_is_the_host_named() {
    let error = BackupError::BadHost(String::new());
    assert_eq!(
        backup_error_report("restore-backup", &error, "origin"),
        refused(
            "restore-backup",
            "host \"\" cannot name a backup namespace: it is empty or holds '/'",
            "the host named",
            "name the host as its backup is named on the remote (refs/mortar-backup/<host>/): mr restore-backup \
             <host> --actor <name>",
        )
    );
}

#[test]
fn git_is_a_failure_naming_the_remote_for_both_verbs() {
    let error = BackupError::Git(GitError::refusal(vec!["ls-remote".to_string()], "fatal: gitio text"));
    assert_eq!(
        backup_error_report("backup", &error, "origin"),
        failed("backup", "git failed while reading the refs or talking to the remote origin")
    );
    assert_eq!(
        backup_error_report("restore-backup", &error, "backup-host"),
        failed("restore-backup", "git failed while reading the refs or talking to the remote backup-host")
    );
}

#[test]
fn a_stale_lease_is_refused_and_the_next_step_repeats() {
    assert_eq!(
        push_refused(PushRefusal::StaleLease),
        refused(
            "backup",
            "the remote origin refused to move refs/mortar-backup/build-mac-02/heads/zone/cli: it moved on the \
             remote since it was listed",
            "the remote origin",
            "repeat the same command; it lists the backup afresh",
        )
    );
}

#[test]
fn a_non_fast_forward_is_refused_and_the_next_step_clears_the_remote() {
    assert_eq!(
        push_refused(PushRefusal::NonFastForward),
        refused(
            "backup",
            "the remote origin refused to move refs/mortar-backup/build-mac-02/heads/zone/cli: the move is not a \
             fast-forward",
            "the remote origin",
            "clear what the remote origin refuses for refs/mortar-backup/, then repeat the same command",
        )
    );
}

#[test]
fn a_hook_is_refused_and_the_next_step_clears_the_remote() {
    assert_eq!(
        push_refused(PushRefusal::Hook),
        refused(
            "backup",
            "the remote origin refused to move refs/mortar-backup/build-mac-02/heads/zone/cli: a hook on the remote \
             declined it",
            "the remote origin",
            "clear what the remote origin refuses for refs/mortar-backup/, then repeat the same command",
        )
    );
}

#[test]
fn another_refusal_quotes_gits_summary_and_the_next_step_clears_the_remote() {
    assert_eq!(
        push_refused(PushRefusal::Other("[remote rejected] (unpacker error)".to_string())),
        refused(
            "backup",
            "the remote origin refused to move refs/mortar-backup/build-mac-02/heads/zone/cli: git's summary was \
             [remote rejected] (unpacker error)",
            "the remote origin",
            "clear what the remote origin refuses for refs/mortar-backup/, then repeat the same command",
        )
    );
}

#[test]
fn a_refusal_on_restore_backup_names_that_verb_and_its_remote() {
    let error = BackupError::Refused { refname: BACKUP_REF.to_string(), refusal: PushRefusal::Hook };
    assert_eq!(
        backup_error_report("restore-backup", &error, "backup-host"),
        refused(
            "restore-backup",
            "the remote backup-host refused to move refs/mortar-backup/build-mac-02/heads/zone/cli: a hook on the \
             remote declined it",
            "the remote backup-host",
            "clear what the remote backup-host refuses for refs/mortar-backup/, then repeat the same command",
        )
    );
}

#[test]
fn oplog_is_a_failure_in_the_clis_words() {
    let error = BackupError::Oplog(OplogError::Malformed { line: Some(3), reason: "oplog text".to_string() });
    assert_eq!(
        backup_error_report("restore-backup", &error, "origin"),
        failed("restore-backup", "the op log could not be read or written")
    );
}

#[test]
fn a_failed_op_is_refused_naming_its_op_id() {
    let error = BackupError::OpFailed(record(OpPhase::Failed));
    assert_eq!(
        backup_error_report("restore-backup", &error, "origin"),
        refused(
            "restore-backup",
            "op op-7 moved nothing: a local ref was not at its expected value",
            "the local refs",
            "repeat the same command; it reads the refs afresh",
        )
    );
}

#[test]
fn a_repeated_op_is_refused_naming_its_key() {
    let error = BackupError::OpRepeated(record(OpPhase::Done));
    assert_eq!(
        backup_error_report("restore-backup", &error, "origin"),
        refused(
            "restore-backup",
            "op key land-restore-backup:build-mac-02:op-7 is already done in the op log, so nothing moved",
            "the op log",
            "repeat the same command",
        )
    );
}
