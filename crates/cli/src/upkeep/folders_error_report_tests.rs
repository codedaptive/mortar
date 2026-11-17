//! folders_error_report: a FolderError from disk or sweep, under the verb it came from. Lock is lock_report's; Git and
//! Oplog are failures in the cli's own words, never gitio's or oplog's Display; DiskProbe names the path and reason;
//! ExtraRefRefused names the --ref to drop (S-112); every other arm is folder_error_why's lines with folder_refusal's
//! exit and next.

use std::path::PathBuf;

use folders::record::RecordError;
use folders::FolderError;
use gitio::GitError;
use oplog::{LockError, LockRecord, OplogError};

use super::folders_error_report;
use crate::report::{ExitCode, OutValue, Report};

fn report(verb: &str, exit: ExitCode, outcome: &str, lines: &[&str], next: &str) -> Report {
    Report {
        verb: verb.to_string(),
        exit,
        outcome: outcome.to_string(),
        lines: lines.iter().map(ToString::to_string).collect(),
        owner: None,
        next: next.to_string(),
        fields: Vec::new(),
    }
}

fn refused(verb: &str, lines: &[&str], next: &str) -> Report {
    report(verb, ExitCode::Refused, "refused", lines, next)
}

fn failed(verb: &str, what: &str) -> Report {
    report(verb, ExitCode::Failure, "failed", &[what, "nothing half-done is left"], "repeat the same command")
}

fn git_error() -> GitError {
    GitError {
        args: vec!["update-ref".to_string(), "--stdin".to_string()],
        code: Some(128),
        stderr: "fatal: gitio text 5521".to_string(),
    }
}

#[test]
fn an_extra_ref_refused_names_the_ref_to_drop_from_the_same_command() {
    let error =
        FolderError::ExtraRefRefused { refname: "refs/heads/main".to_string(), reason: "is protected".to_string() };
    assert_eq!(
        folders_error_report("sweep", &error),
        refused(
            "sweep",
            &["extra ref refs/heads/main is protected"],
            "mr sweep with the same words without --ref refs/heads/main, with --actor <name>",
        )
    );
}

#[test]
fn an_extra_ref_refused_names_the_verb_it_came_from() {
    let error = FolderError::ExtraRefRefused {
        refname: "refs/t3/x".to_string(),
        reason: "is not a full ref name".to_string(),
    };
    assert_eq!(
        folders_error_report("disk", &error).next,
        "mr disk with the same words without --ref refs/t3/x, with --actor <name>"
    );
}

#[test]
fn a_ref_moved_is_refused_and_repeated() {
    let error = FolderError::RefMoved { refname: "refs/mortar/replayed/core/x".to_string(), at: None };
    assert_eq!(
        folders_error_report("sweep", &error),
        refused(
            "sweep",
            &["ref refs/mortar/replayed/core/x moved while it was being deleted; it is now at no commit; nothing was \
               deleted"],
            "repeat the same command",
        )
    );
}

#[test]
fn io_is_a_failure_with_its_path_and_folder_refusals_next() {
    let error = FolderError::Io { path: PathBuf::from("/w/a"), source: std::io::Error::other("disk gone") };
    assert_eq!(
        folders_error_report("disk", &error),
        report(
            "disk",
            ExitCode::Failure,
            "failed",
            &["/w/a: disk gone"],
            "repeat the same command; nothing half-done is left",
        )
    );
}

#[test]
fn a_record_mortar_does_not_write_is_refused_with_its_reason() {
    let error = FolderError::Record {
        path: PathBuf::from("/w/a/.git"),
        error: RecordError { reason: "no key kind".to_string() },
    };
    assert_eq!(
        folders_error_report("disk", &error),
        refused(
            "disk",
            &["/w/a/.git: the folder record is not one Mortar writes: no key kind"],
            "make the folder again at another path with mr folder zone <zone> <path> --actor <name> or mr \
             folder request <zone> <request> <path> --paths <path>... --actor <name>",
        )
    );
}

#[test]
fn a_missing_zone_branch_is_refused_with_folder_refusals_next() {
    let error = FolderError::ZoneBranchMissing { branch: "zone/core".to_string() };
    assert_eq!(
        folders_error_report("sweep", &error),
        refused(
            "sweep",
            &["zone branch zone/core does not exist"],
            "mr folder zone <zone> <path> --start <commit> --actor <name>, then repeat",
        )
    );
}

#[test]
fn git_is_a_failure_in_the_clis_words() {
    assert_eq!(
        folders_error_report("sweep", &FolderError::Git(git_error())),
        failed("sweep", "mr sweep: a git run failed")
    );
}

#[test]
fn git_never_prints_gitios_text() {
    for verb in ["disk", "sweep"] {
        let printed = format!("{:?}", folders_error_report(verb, &FolderError::Git(git_error())));
        assert!(!printed.contains("5521"), "{printed}");
        assert!(!printed.contains("update-ref"), "{printed}");
    }
}

#[test]
fn oplog_is_a_failure_in_the_clis_words() {
    let error = FolderError::Oplog(OplogError::Malformed { line: None, reason: "no op_id".to_string() });
    let report = folders_error_report("disk", &error);
    assert_eq!(report, failed("disk", "mr disk: the op log could not be read or written"));
    assert!(!format!("{report:?}").contains("malformed op data"), "{report:?}");
}

#[test]
fn disk_probe_is_a_failure_naming_the_path_and_reason() {
    let error = FolderError::DiskProbe { path: PathBuf::from("/w"), reason: "df failed".to_string() };
    assert_eq!(
        folders_error_report("disk", &error),
        failed("disk", "mr disk: the free disk at /w could not be read: df failed")
    );
}

#[test]
fn a_held_lock_is_busy_as_lock_report_words_it() {
    let error = FolderError::Lock(LockError::Held {
        path: PathBuf::from("/r/.git/mortar/locks/folders.lock"),
        holder: LockRecord {
            pid: 4242,
            host: "build-7".to_string(),
            key: "sweep-a1b2".to_string(),
            since: 1_763_100_000,
        },
        removed_guards: Vec::new(),
    });
    assert_eq!(
        folders_error_report("sweep", &error),
        Report {
            verb: "sweep".to_string(),
            exit: ExitCode::Busy,
            outcome: "busy".to_string(),
            lines: vec!["the lock /r/.git/mortar/locks/folders.lock is held by pid 4242 on build-7 for op sweep-a1b2 \
                         since 1763100000 (unix seconds); nothing ran"
                .to_string()],
            owner: Some("pid 4242 on build-7".to_string()),
            next: "repeat the same command later".to_string(),
            fields: vec![
                ("holder_pid".to_string(), OutValue::Number(4242)),
                ("holder_host".to_string(), OutValue::Text("build-7".to_string())),
                ("holder_key".to_string(), OutValue::Text("sweep-a1b2".to_string())),
                ("since".to_string(), OutValue::Number(1_763_100_000)),
            ],
        }
    );
}
