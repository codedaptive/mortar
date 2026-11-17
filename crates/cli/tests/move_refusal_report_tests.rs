//! move_refusal_report: files in the way, a branch elsewhere and every other variant folder_refusal refuses are exit 1
//! owned by the folder, after the lines of what was already done; a lock is lock_report's report and a git, op log or
//! disk probe error a failure, with those lines first. folders' own Display text is never printed. ZIT (S-79 as
//! replaced: the line is the file): it calls report::failure_report, lock::lock_report, folder::folder_refusal and
//! folder::folder_error_why for real.

use std::path::PathBuf;

use folders::FolderError;
use gitio::Oid;
use oplog::LockError;

use cli::{move_refusal_report, ExitCode, Report};

const OLD: &str = "1111111111111111111111111111111111111111";
const NEW: &str = "2222222222222222222222222222222222222222";
const ELSE: &str = "3333333333333333333333333333333333333333";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn before() -> Vec<String> {
    vec!["rebased: refs/heads/req/core/fix-1 moved".to_string()]
}

#[test]
fn files_in_the_way_are_listed_and_the_next_join_finishes() {
    let error = FolderError::MoveRefused {
        path: PathBuf::from("/w/fix-1"),
        paths: vec![PathBuf::from("src/a.rs"), PathBuf::from("b.txt")],
    };
    assert_eq!(
        move_refusal_report(&error, before()),
        Report {
            verb: "join".to_string(),
            exit: ExitCode::Refused,
            outcome: "refused".to_string(),
            lines: vec![
                "rebased: refs/heads/req/core/fix-1 moved".to_string(),
                "the folder could not move; these files stand in the way:".to_string(),
                "  src/a.rs".to_string(),
                "  b.txt".to_string(),
                "the request branch already stands at the new tip; the next mr join, mr resolve, mr save or mr undo finishes the move first".to_string(),
            ],
            owner: Some("this request's folder".to_string()),
            next: "move those files out of the way, then mr join".to_string(),
            fields: Vec::new(),
        }
    );
}

#[test]
fn a_branch_elsewhere_is_put_back() {
    let error = FolderError::BranchElsewhere {
        branch: "refs/heads/req/core/fix-1".to_string(),
        at: Some(oid(ELSE)),
        old: oid(OLD),
        new: oid(NEW),
    };
    assert_eq!(
        move_refusal_report(&error, Vec::new()),
        Report {
            verb: "join".to_string(),
            exit: ExitCode::Refused,
            outcome: "refused".to_string(),
            lines: vec!["branch refs/heads/req/core/fix-1 is at 3333333333333333333333333333333333333333, neither \
                         the move's old tip 1111111111111111111111111111111111111111 nor its new tip \
                         2222222222222222222222222222222222222222"
                .to_string()],
            owner: Some("this request's folder".to_string()),
            next: "put the request branch back where mortar left it, then mr join".to_string(),
            fields: Vec::new(),
        }
    );
}

#[test]
fn a_folders_refusal_is_refused_with_folder_refusals_next_after_the_lines_before() {
    let error = FolderError::NotSaved { path: PathBuf::from("/w/fix-1") };
    assert_eq!(
        move_refusal_report(&error, before()),
        Report {
            verb: "join".to_string(),
            exit: ExitCode::Refused,
            outcome: "refused".to_string(),
            lines: vec![
                "rebased: refs/heads/req/core/fix-1 moved".to_string(),
                "/w/fix-1 holds work that is not saved".to_string(),
            ],
            owner: Some("this request's folder".to_string()),
            next: "repeat once the folder's work is saved".to_string(),
            fields: Vec::new(),
        }
    );
}

#[test]
fn a_branch_at_no_commit_says_no_commit() {
    let error = FolderError::BranchElsewhere {
        branch: "refs/heads/req/core/fix-1".to_string(),
        at: None,
        old: oid(OLD),
        new: oid(NEW),
    };
    assert_eq!(
        move_refusal_report(&error, before()),
        Report {
            verb: "join".to_string(),
            exit: ExitCode::Refused,
            outcome: "refused".to_string(),
            lines: vec![
                "rebased: refs/heads/req/core/fix-1 moved".to_string(),
                "branch refs/heads/req/core/fix-1 is at no commit, neither the move's old tip \
                 1111111111111111111111111111111111111111 nor its new tip 2222222222222222222222222222222222222222"
                    .to_string(),
            ],
            owner: Some("this request's folder".to_string()),
            next: "put the request branch back where mortar left it, then mr join".to_string(),
            fields: Vec::new(),
        }
    );
}

#[test]
fn a_folders_variant_is_refused_with_the_cli_line_not_folders_text() {
    let error = FolderError::NotAFolder { path: PathBuf::from("/w/fix-1"), reason: "no record".to_string() };
    assert_eq!(
        move_refusal_report(&error, before()),
        Report {
            verb: "join".to_string(),
            exit: ExitCode::Refused,
            outcome: "refused".to_string(),
            lines: vec![
                "rebased: refs/heads/req/core/fix-1 moved".to_string(),
                "/w/fix-1 is in the way and is not a Mortar folder: no record".to_string(),
            ],
            owner: Some("this request's folder".to_string()),
            next: "choose another path, or move what is at that path out of the way, then repeat".to_string(),
            fields: Vec::new(),
        }
    );
}

#[test]
fn a_free_disk_that_cannot_be_read_fails_with_its_path_and_reason() {
    let error = FolderError::DiskProbe { path: PathBuf::from("/w"), reason: "statfs failed".to_string() };
    assert_eq!(
        move_refusal_report(&error, before()),
        Report {
            verb: "join".to_string(),
            exit: ExitCode::Failure,
            outcome: "failed".to_string(),
            lines: vec![
                "rebased: refs/heads/req/core/fix-1 moved".to_string(),
                "the free disk at /w could not be read: statfs failed".to_string(),
                "nothing half-done is left".to_string(),
            ],
            owner: None,
            next: "repeat the same command".to_string(),
            fields: Vec::new(),
        }
    );
}

#[test]
fn a_lock_that_kept_changing_hands_is_busy_after_the_lines_before() {
    let error =
        FolderError::Lock(LockError::Contended { path: PathBuf::from("/r/folders.lock"), removed_guards: Vec::new() });
    assert_eq!(
        move_refusal_report(&error, before()),
        Report {
            verb: "join".to_string(),
            exit: ExitCode::Busy,
            outcome: "busy".to_string(),
            lines: vec![
                "rebased: refs/heads/req/core/fix-1 moved".to_string(),
                "the lock /r/folders.lock kept changing hands; nothing ran".to_string(),
            ],
            owner: None,
            next: "repeat the same command later".to_string(),
            fields: Vec::new(),
        }
    );
}
