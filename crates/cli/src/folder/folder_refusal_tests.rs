//! folder_refusal: the exit code and next step of each FolderError variant, built as data; Lock, Git, Oplog and
//! DiskProbe are left to the cli's lock and failure reports.

use std::path::PathBuf;

use folders::plan::{BelowFloor, PlanError};
use folders::record::RecordError;
use folders::{FolderError, FolderKind, FolderRecord, SweepEstimate};
use gitio::{GitError, Oid};
use oplog::{LockError, OplogError};

use super::folder_refusal;
use crate::report::ExitCode;

fn oid(c: char) -> Oid {
    Oid::parse(&c.to_string().repeat(40)).expect("a valid oid")
}

fn path(text: &str) -> PathBuf {
    PathBuf::from(text)
}

fn refused(error: FolderError, next: &str) {
    assert_eq!(folder_refusal(&error), Some((ExitCode::Refused, next.to_string())), "{error:?}");
}

#[test]
fn lock_is_left_to_lock_report() {
    assert_eq!(folder_refusal(&FolderError::Lock(LockError::BadName("a b".to_string()))), None);
}

#[test]
fn git_is_left_to_failure_report() {
    let git = GitError { args: vec!["worktree".to_string()], code: Some(128), stderr: "fatal".to_string() };
    assert_eq!(folder_refusal(&FolderError::Git(git)), None);
}

#[test]
fn oplog_is_left_to_failure_report() {
    let oplog = OplogError::NotOpen { op_id: "op-1".to_string() };
    assert_eq!(folder_refusal(&FolderError::Oplog(oplog)), None);
}

#[test]
fn disk_probe_is_left_to_failure_report() {
    let error = FolderError::DiskProbe { path: path("/w"), reason: "df failed".to_string() };
    assert_eq!(folder_refusal(&error), None);
}

#[test]
fn io_is_a_failure_whose_repeat_is_safe() {
    let error = FolderError::Io { path: path("/w/a"), source: std::io::Error::other("disk gone") };
    assert_eq!(
        folder_refusal(&error),
        Some((ExitCode::Failure, "repeat the same command".to_string()))
    );
}

#[test]
fn plan() {
    refused(
        FolderError::Plan(PlanError { reason: "bad name".to_string() }),
        "repeat with names and paths the folder plan takes",
    );
}

#[test]
fn record() {
    refused(
        FolderError::Record { path: path("/w/a"), error: RecordError { reason: "no kind".to_string() } },
        "make the folder again at another path with mr folder zone <zone> <path> --actor <name> or mr folder \
         request <zone> <request> <path> --paths <path>... --actor <name>",
    );
}

#[test]
fn below_floor() {
    let below = BelowFloor { free: 1, floor: 2, sweep: SweepEstimate { bytes: 0, reason: None } };
    refused(FolderError::BelowFloor(below), "free disk space until it is over the floor, then repeat");
}

#[test]
fn different_folder() {
    let existing = FolderRecord {
        kind: FolderKind::Zone,
        zone: "core".to_string(),
        request: None,
        base: oid('a'),
        work_order_paths: Vec::new(),
        needed_paths: Vec::new(),
        parent: None,
    };
    refused(
        FolderError::DifferentFolder { path: path("/w/a"), existing },
        "choose another path for the folder, then repeat",
    );
}

#[test]
fn not_a_folder() {
    refused(
        FolderError::NotAFolder { path: path("/w/a"), reason: "no record".to_string() },
        "choose another path, or move what is at that path out of the way, then repeat",
    );
}

#[test]
fn branch_checked_out() {
    refused(
        FolderError::BranchCheckedOut { branch: "zone/core".to_string(), path: path("/w/z") },
        "use the folder that has the branch checked out, or remove it, then repeat",
    );
}

#[test]
fn branch_exists() {
    refused(
        FolderError::BranchExists { branch: "req/core/fix-1".to_string() },
        "choose another request name, then repeat",
    );
}

#[test]
fn zone_branch_missing() {
    refused(
        FolderError::ZoneBranchMissing { branch: "zone/core".to_string() },
        "mr folder zone <zone> <path> --start <commit> --actor <name>, then repeat",
    );
}

#[test]
fn request_branch_missing() {
    refused(
        FolderError::RequestBranchMissing { branch: "req/core/fix-1".to_string() },
        "mr folder request <zone> <request> <path> --paths <path>... --actor <name>, then repeat",
    );
}

#[test]
fn attempt_missing() {
    refused(
        FolderError::AttemptMissing { branch: "try/core/fix-1/3".to_string() },
        "mr folder keep <k> --actor <name> with an attempt that exists, or mr folder drop --actor <name>",
    );
}

#[test]
fn attempts_unsaved() {
    refused(
        FolderError::AttemptsUnsaved { paths: vec![path("/w/tries/try-1")] },
        "repeat once each listed attempt's work is saved",
    );
}

#[test]
fn attempts_remain() {
    refused(
        FolderError::AttemptsRemain { path: path("/w/fix"), attempts: vec!["refs/heads/try/core/fix-1/1".to_string()] },
        "mr folder keep <k> --actor <name> or mr folder drop --actor <name>, run in the request's folder",
    );
}

#[test]
fn orphan_attempt() {
    refused(FolderError::OrphanAttempt { paths: vec![path("/w/tries/try-1")] }, "repeat the same command");
}

#[test]
fn orphan_without_snapshot() {
    refused(
        FolderError::OrphanWithoutSnapshot { paths: vec![path("/w/tries/try-1")] },
        "copy out anything wanted from the listed folders, remove each by hand, then repeat",
    );
}

#[test]
fn request_moved_on() {
    refused(
        FolderError::RequestMovedOn { branch: "req/core/fix-1".to_string(), attempt: "try/core/fix-1/2".to_string() },
        "bring the request branch's commits into the attempt, or make the attempts again, then repeat",
    );
}

#[test]
fn attempt_folder() {
    refused(
        FolderError::AttemptFolder { path: path("/w/tries/try-2"), k: 2 },
        "after keep or drop, run mr folder remove <path> --actor <name> with the request's own folder as <path>",
    );
}

#[test]
fn not_saved() {
    refused(FolderError::NotSaved { path: path("/w/fix") }, "repeat once the folder's work is saved");
}

#[test]
fn move_refused() {
    refused(
        FolderError::MoveRefused { path: path("/w/fix"), paths: vec![path("src/a.rs")] },
        "move the local changes at the listed paths out of the way, then repeat",
    );
}

#[test]
fn branch_elsewhere() {
    refused(
        FolderError::BranchElsewhere { branch: "req/core/fix-1".to_string(), at: None, old: oid('a'), new: oid('b') },
        "read who moved the branch with mr log --ref <full ref name>, then repeat",
    );
}

#[test]
fn not_removed_unsaved() {
    refused(
        FolderError::NotRemovedUnsaved { path: path("/w/fix"), refused: vec![path("big.bin")], cap: Some(10) },
        "repeat once the folder's work is saved",
    );
}

#[test]
fn not_joined() {
    refused(
        FolderError::NotJoined {
            path: path("/w/fix"),
            branch: "refs/heads/req/core/fix-1".to_string(),
            zone_branch: "refs/heads/zone/core".to_string(),
        },
        "repeat once the request has joined its zone",
    );
}

#[test]
fn zone_folder() {
    refused(
        FolderError::ZoneFolder { path: path("/w/z"), zone: "core".to_string() },
        "leave the zone folder in place; mr folder remove <path> --actor <name> takes request folders only",
    );
}

#[test]
fn ref_moved() {
    refused(
        FolderError::RefMoved { refname: "refs/heads/try/core/fix-1/1".to_string(), at: Some(oid('c')) },
        "repeat the same command",
    );
}

#[test]
fn attempt_refs_left() {
    refused(
        FolderError::AttemptRefsLeft {
            moved: "refs/heads/try/core/fix-1/1".to_string(),
            at: None,
            removed: Vec::new(),
        },
        "repeat the same command",
    );
}

#[test]
fn removed_refs_left() {
    refused(
        FolderError::RemovedRefsLeft {
            path: path("/w/fix"),
            branch: "refs/heads/req/core/fix-1".to_string(),
            moved: "refs/mortar/snap/core/fix-1".to_string(),
            at: Some(oid('d')),
        },
        "nothing to do now: the refs hold only work already in the zone",
    );
}

#[test]
fn extra_ref_refused() {
    refused(
        FolderError::ExtraRefRefused { refname: "refs/heads/main".to_string(), reason: "is protected".to_string() },
        "repeat without that ref",
    );
}
