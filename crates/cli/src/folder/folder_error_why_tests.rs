//! folder_error_why: the facts of each FolderError variant, built as data, with no next step; none for the variants
//! the cli's lock and failure reports answer.

use std::path::PathBuf;

use folders::plan::{BelowFloor, PlanError};
use folders::record::RecordError;
use folders::{FolderError, FolderKind, FolderRecord, SweepEstimate};
use gitio::{GitError, Oid};
use oplog::{LockError, OplogError};

use super::folder_error_why;

const A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const C: &str = "cccccccccccccccccccccccccccccccccccccccc";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).expect("a valid oid")
}

fn path(text: &str) -> PathBuf {
    PathBuf::from(text)
}

fn why(error: FolderError, lines: &[&str]) {
    let expected: Vec<String> = lines.iter().map(ToString::to_string).collect();
    assert_eq!(folder_error_why(&error), expected, "{error:?}");
}

fn record(kind: FolderKind, request: Option<&str>) -> FolderRecord {
    FolderRecord {
        kind,
        zone: "core".to_string(),
        request: request.map(str::to_string),
        base: oid(A),
        work_order_paths: Vec::new(),
        needed_paths: Vec::new(),
        parent: None,
    }
}

#[test]
fn lock_git_oplog_and_disk_probe_give_no_lines() {
    why(FolderError::Lock(LockError::HostUnknown("no host".to_string())), &[]);
    why(FolderError::Git(GitError { args: Vec::new(), code: None, stderr: "killed".to_string() }), &[]);
    why(FolderError::Oplog(OplogError::UnknownOp { op_id: "op-9".to_string() }), &[]);
    why(FolderError::DiskProbe { path: path("/w"), reason: "df failed".to_string() }, &[]);
}

#[test]
fn plan_gives_its_reason() {
    why(
        FolderError::Plan(PlanError { reason: "zone name \"a b\" holds a space".to_string() }),
        &["folder plan: zone name \"a b\" holds a space"],
    );
}

#[test]
fn record_gives_its_path_and_the_record_reason() {
    why(
        FolderError::Record { path: path("/w/a/.git"), error: RecordError { reason: "no key kind".to_string() } },
        &["/w/a/.git: the folder record is not one Mortar writes: no key kind"],
    );
}

#[test]
fn io_gives_its_path_and_the_io_text() {
    why(
        FolderError::Io { path: path("/w/a"), source: std::io::Error::other("disk gone") },
        &["/w/a: disk gone"],
    );
}

#[test]
fn below_floor_gives_free_and_floor_and_never_a_sweep_figure() {
    let sweep = SweepEstimate { bytes: 0, reason: Some("not measured by mr folder".to_string()) };
    why(
        FolderError::BelowFloor(BelowFloor { free: 100, floor: 5000, sweep }),
        &["free disk 100 bytes is under the floor of 5000 bytes; nothing was made"],
    );
}

#[test]
fn different_folder_names_the_folder_already_there() {
    why(
        FolderError::DifferentFolder { path: path("/w/a"), existing: record(FolderKind::Zone, None) },
        &["/w/a is already the zone folder of zone core, not the folder asked for"],
    );
    why(
        FolderError::DifferentFolder { path: path("/w/a"), existing: record(FolderKind::Request, Some("fix-1")) },
        &["/w/a is already the request folder of request fix-1 in zone core, not the folder asked for"],
    );
    why(
        FolderError::DifferentFolder { path: path("/w/a"), existing: record(FolderKind::Attempt(2), Some("fix-1")) },
        &["/w/a is already attempt folder 2 of request fix-1 in zone core, not the folder asked for"],
    );
}

#[test]
fn not_a_folder() {
    why(
        FolderError::NotAFolder { path: path("/w/a"), reason: "a file is there".to_string() },
        &["/w/a is in the way and is not a Mortar folder: a file is there"],
    );
}

#[test]
fn branch_checked_out() {
    why(
        FolderError::BranchCheckedOut { branch: "zone/core".to_string(), path: path("/w/z") },
        &["branch zone/core is checked out in the folder at /w/z"],
    );
}

#[test]
fn branch_exists() {
    why(
        FolderError::BranchExists { branch: "req/core/fix-1".to_string() },
        &["branch req/core/fix-1 already exists at another commit, or Mortar did not make it at this one"],
    );
}

#[test]
fn zone_request_and_attempt_branch_missing() {
    why(FolderError::ZoneBranchMissing { branch: "zone/core".to_string() }, &["zone branch zone/core does not exist"]);
    why(
        FolderError::RequestBranchMissing { branch: "req/core/fix-1".to_string() },
        &["request branch req/core/fix-1 does not exist"],
    );
    why(
        FolderError::AttemptMissing { branch: "try/core/fix-1/3".to_string() },
        &["attempt branch try/core/fix-1/3 does not exist"],
    );
}

#[test]
fn attempts_unsaved_names_each_folder_or_branch() {
    why(
        FolderError::AttemptsUnsaved { paths: vec![path("/w/tries/try-1"), path("refs/heads/try/core/fix-1/2")] },
        &[
            "the work of /w/tries/try-1 is not saved",
            "the work of refs/heads/try/core/fix-1/2 is not saved",
            "nothing changed",
        ],
    );
}

#[test]
fn attempts_remain() {
    why(
        FolderError::AttemptsRemain {
            path: path("/w/fix"),
            attempts: vec!["refs/heads/try/core/fix-1/1".to_string(), "/w/tries/try-2".to_string()],
        },
        &["attempts of the request at /w/fix remain: refs/heads/try/core/fix-1/1, /w/tries/try-2", "nothing changed"],
    );
}

#[test]
fn orphan_attempt() {
    why(
        FolderError::OrphanAttempt { paths: vec![path("/w/tries/try-1")] },
        &["the attempt folder /w/tries/try-1 has lost its try branch", "nothing changed"],
    );
}

#[test]
fn orphan_without_snapshot() {
    why(
        FolderError::OrphanWithoutSnapshot { paths: vec![path("/w/tries/try-1")] },
        &[
            "the attempt folder /w/tries/try-1 has lost its try branch, and Mortar holds no copy of it",
            "nothing changed",
        ],
    );
}

#[test]
fn request_moved_on() {
    why(
        FolderError::RequestMovedOn { branch: "req/core/fix-1".to_string(), attempt: "try/core/fix-1/2".to_string() },
        &["request branch req/core/fix-1 has commits attempt branch try/core/fix-1/2 does not hold", "nothing changed"],
    );
}

#[test]
fn attempt_folder() {
    why(
        FolderError::AttemptFolder { path: path("/w/tries/try-2"), k: 2 },
        &["/w/tries/try-2 is attempt folder 2; keep and drop remove attempt folders", "nothing changed"],
    );
}

#[test]
fn not_saved() {
    why(FolderError::NotSaved { path: path("/w/fix") }, &["/w/fix holds work that is not saved"]);
}

#[test]
fn move_refused() {
    why(
        FolderError::MoveRefused { path: path("/w/fix"), paths: vec![path("src/a.rs"), path("notes.txt")] },
        &["/w/fix was not moved: local changes are in the way at src/a.rs, notes.txt", "nothing changed"],
    );
}

#[test]
fn branch_elsewhere_with_and_without_a_commit() {
    why(
        FolderError::BranchElsewhere {
            branch: "req/core/fix-1".to_string(),
            at: Some(oid(C)),
            old: oid(A),
            new: oid(B),
        },
        &[&format!("branch req/core/fix-1 is at {C}, neither the move's old tip {A} nor its new tip {B}")],
    );
    why(
        FolderError::BranchElsewhere { branch: "req/core/fix-1".to_string(), at: None, old: oid(A), new: oid(B) },
        &[&format!("branch req/core/fix-1 is at no commit, neither the move's old tip {A} nor its new tip {B}")],
    );
}

#[test]
fn not_removed_unsaved_without_refused_files() {
    why(
        FolderError::NotRemovedUnsaved { path: path("/w/fix"), refused: Vec::new(), cap: Some(10) },
        &["/w/fix holds work that is not saved"],
    );
}

#[test]
fn not_removed_unsaved_names_the_refused_files_and_the_cap() {
    why(
        FolderError::NotRemovedUnsaved {
            path: path("/w/fix"),
            refused: vec![path("a.bin"), path("b.bin")],
            cap: Some(10),
        },
        &["/w/fix holds files the snapshot size cap of 10 bytes refused: a.bin, b.bin; they are in no snapshot"],
    );
    why(
        FolderError::NotRemovedUnsaved { path: path("/w/fix"), refused: vec![path("a.bin")], cap: None },
        &["/w/fix holds files the snapshot size cap refused: a.bin; they are in no snapshot"],
    );
}

#[test]
fn not_joined() {
    why(
        FolderError::NotJoined {
            path: path("/w/fix"),
            branch: "refs/heads/req/core/fix-1".to_string(),
            zone_branch: "refs/heads/zone/core".to_string(),
        },
        &["branch refs/heads/req/core/fix-1 of /w/fix is not contained in refs/heads/zone/core"],
    );
}

#[test]
fn zone_folder() {
    why(
        FolderError::ZoneFolder { path: path("/w/z"), zone: "core".to_string() },
        &["/w/z is the zone folder of zone core", "nothing changed"],
    );
}

#[test]
fn ref_moved() {
    why(
        FolderError::RefMoved { refname: "refs/heads/try/core/fix-1/1".to_string(), at: None },
        &["ref refs/heads/try/core/fix-1/1 moved while it was being deleted; it is now at no commit; nothing was \
           deleted"],
    );
}

#[test]
fn attempt_refs_left_names_the_folders_removed() {
    why(
        FolderError::AttemptRefsLeft {
            moved: "refs/heads/try/core/fix-1/1".to_string(),
            at: Some(oid(C)),
            removed: vec![path("/w/tries/try-1")],
        },
        &[
            &format!(
                "ref refs/heads/try/core/fix-1/1 moved before the op ran; it is now at {C}; every attempt branch is \
                 kept"
            ),
            "the attempt folder /w/tries/try-1 was removed",
        ],
    );
}

#[test]
fn removed_refs_left() {
    why(
        FolderError::RemovedRefsLeft {
            path: path("/w/fix"),
            branch: "refs/heads/req/core/fix-1".to_string(),
            moved: "refs/mortar/snap/core/fix-1".to_string(),
            at: Some(oid(C)),
        },
        &[
            "/w/fix was removed",
            &format!(
                "ref refs/mortar/snap/core/fix-1 moved (now at {C}) before branch refs/heads/req/core/fix-1 and its \
                 snapshot ref were deleted; both are left"
            ),
        ],
    );
}

#[test]
fn extra_ref_refused() {
    why(
        FolderError::ExtraRefRefused { refname: "refs/heads/main".to_string(), reason: "is protected".to_string() },
        &["extra ref refs/heads/main is protected"],
    );
}

#[test]
fn no_line_holds_a_next_step() {
    let errors = [
        FolderError::NotRemovedUnsaved { path: path("/w/fix"), refused: Vec::new(), cap: None },
        FolderError::NotJoined { path: path("/w/fix"), branch: "b".to_string(), zone_branch: "z".to_string() },
        FolderError::AttemptsUnsaved { paths: vec![path("/w/t")] },
        FolderError::RemovedRefsLeft {
            path: path("/w/fix"),
            branch: "b".to_string(),
            moved: "m".to_string(),
            at: None,
        },
    ];
    for error in errors {
        for line in folder_error_why(&error) {
            assert!(!line.contains("next step") && !line.contains("mortar ") && !line.contains("mr "), "{line}");
        }
    }
}
