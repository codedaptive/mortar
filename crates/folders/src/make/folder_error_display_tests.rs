//! The text of each FolderError variant.

use std::io::ErrorKind;
use std::path::PathBuf;

use gitio::{GitError, Oid};
use oplog::{LockError, OplogError};

use super::FolderError;
use crate::plan::{BelowFloor, PlanError, SweepEstimate};
use crate::record::{FolderKind, FolderRecord, RecordError};

fn text(err: FolderError) -> String {
    err.to_string()
}

fn existing(kind: FolderKind, request: Option<&str>) -> FolderRecord {
    FolderRecord {
        kind,
        zone: "folders".to_string(),
        request: request.map(str::to_string),
        base: Oid::parse("489d82745705a031306253da29bdad7aee19d2e5").unwrap(),
        work_order_paths: vec!["crates/folders".to_string()],
        needed_paths: vec![],
        parent: None,
    }
}

#[test]
fn plan_reads_as_the_plan_refusal() {
    let err = FolderError::Plan(PlanError {
        reason: "zone name \"\" is empty".to_string(),
    });
    assert_eq!(text(err), "folder plan: zone name \"\" is empty");
}

#[test]
fn record_names_the_file_and_the_reason() {
    let err = FolderError::Record {
        path: PathBuf::from("/g/mortar-folder.json"),
        error: RecordError {
            reason: "missing key \"kind\"".to_string(),
        },
    };
    assert_eq!(text(err), "/g/mortar-folder.json: folder record: missing key \"kind\"");
}

#[test]
fn git_reads_as_the_git_error() {
    let err = FolderError::Git(GitError {
        args: vec!["worktree".to_string(), "add".to_string()],
        code: Some(128),
        stderr: "fatal: x\n".to_string(),
    });
    assert_eq!(text(err), "git worktree add: exit code 128: fatal: x");
}

#[test]
fn oplog_reads_as_the_oplog_error() {
    let err = FolderError::Oplog(OplogError::Malformed {
        line: None,
        reason: "r".to_string(),
    });
    assert_eq!(text(err), "malformed op data: r; mortar did not write this text, so check what did");
}

#[test]
fn lock_names_the_folders_lock() {
    let err = FolderError::Lock(LockError::Contended {
        path: PathBuf::from("/r/.git/mortar/locks/x.lock"),
        removed_guards: Vec::new(),
    });
    assert_eq!(
        text(err),
        "folders lock: /r/.git/mortar/locks/x.lock changed hands repeatedly while acquiring; try again"
    );
}

#[test]
fn below_floor_says_the_folder_was_not_made() {
    let err = FolderError::BelowFloor(BelowFloor {
        free: 5,
        floor: 10,
        sweep: SweepEstimate {
            bytes: 3,
            reason: None,
        },
    });
    assert_eq!(
        text(err),
        "folder not made: free disk 5 bytes is under the floor of 10 bytes; a sweep would free 3 bytes"
    );
}

#[test]
fn disk_probe_names_the_path_and_reason() {
    let err = FolderError::DiskProbe {
        path: PathBuf::from("/wt"),
        reason: "df failed".to_string(),
    };
    assert_eq!(text(err), "cannot read free disk at /wt: df failed");
}

#[test]
fn different_folder_names_a_request_folder() {
    let err = FolderError::DifferentFolder {
        path: PathBuf::from("/wt/a"),
        existing: existing(FolderKind::Request, Some("WO-1")),
    };
    assert_eq!(
        text(err),
        "/wt/a is already the request folder for zone folders, request WO-1; remove it or choose another path"
    );
}

#[test]
fn different_folder_names_a_zone_folder() {
    let err = FolderError::DifferentFolder {
        path: PathBuf::from("/wt/a"),
        existing: existing(FolderKind::Zone, None),
    };
    assert_eq!(
        text(err),
        "/wt/a is already the zone folder for zone folders; remove it or choose another path"
    );
}

#[test]
fn not_a_folder_names_the_path_and_reason() {
    let err = FolderError::NotAFolder {
        path: PathBuf::from("/wt/a"),
        reason: "exists and is not a worktree".to_string(),
    };
    assert_eq!(text(err), "/wt/a is not a mortar folder: exists and is not a worktree");
}

#[test]
fn branch_checked_out_names_branch_and_holder() {
    let err = FolderError::BranchCheckedOut {
        branch: "zone/folders".to_string(),
        path: PathBuf::from("/wt/b"),
    };
    assert_eq!(text(err), "branch zone/folders is already checked out at /wt/b");
}

#[test]
fn branch_exists_names_the_branch() {
    let err = FolderError::BranchExists {
        branch: "req/folders/WO-1".to_string(),
    };
    assert_eq!(
        text(err),
        "branch req/folders/WO-1 already exists at another commit or was not made by mortar at this one; the folder \
         was not made"
    );
}

#[test]
fn zone_branch_missing_names_the_branch() {
    let err = FolderError::ZoneBranchMissing {
        branch: "zone/folders".to_string(),
    };
    assert_eq!(text(err), "zone branch zone/folders does not exist");
}

#[test]
fn io_names_the_path_and_the_error() {
    let err = FolderError::Io {
        path: PathBuf::from("/g/x"),
        source: std::io::Error::new(ErrorKind::Other, "disk full"),
    };
    assert_eq!(text(err), "/g/x: disk full");
}

#[test]
fn not_saved_names_the_folder_and_the_next_step() {
    let err = FolderError::NotSaved {
        path: PathBuf::from("/wt/a"),
    };
    assert_eq!(
        text(err),
        "/wt/a holds work that is not saved; the folder was not moved; next step: mortar save"
    );
}

#[test]
fn move_refused_names_the_folder_and_every_path_in_the_way() {
    let err = FolderError::MoveRefused {
        path: PathBuf::from("/wt/a"),
        paths: vec![PathBuf::from("src/a.rs"), PathBuf::from("docs/b.md")],
    };
    assert_eq!(
        text(err),
        "/wt/a was not moved: local changes are in the way at src/a.rs, docs/b.md; nothing changed"
    );
}

#[test]
fn branch_elsewhere_names_where_the_branch_is_and_both_tips() {
    let err = FolderError::BranchElsewhere {
        branch: "refs/heads/req/folders/WO-1".to_string(),
        at: Some(Oid::parse("2222222222222222222222222222222222222222").unwrap()),
        old: Oid::parse("489d82745705a031306253da29bdad7aee19d2e5").unwrap(),
        new: Oid::parse("1111111111111111111111111111111111111111").unwrap(),
    };
    assert_eq!(
        text(err),
        "branch refs/heads/req/folders/WO-1 is at 2222222222222222222222222222222222222222, neither the move's old \
         tip 489d82745705a031306253da29bdad7aee19d2e5 nor its new tip 1111111111111111111111111111111111111111"
    );
}

#[test]
fn branch_elsewhere_with_no_commit_says_so() {
    let err = FolderError::BranchElsewhere {
        branch: "refs/heads/req/folders/WO-1".to_string(),
        at: None,
        old: Oid::parse("489d82745705a031306253da29bdad7aee19d2e5").unwrap(),
        new: Oid::parse("1111111111111111111111111111111111111111").unwrap(),
    };
    assert_eq!(
        text(err),
        "branch refs/heads/req/folders/WO-1 is at no commit, neither the move's old tip \
         489d82745705a031306253da29bdad7aee19d2e5 nor its new tip 1111111111111111111111111111111111111111"
    );
}

#[test]
fn not_removed_unsaved_with_no_refused_file_says_save() {
    let err = FolderError::NotRemovedUnsaved {
        path: PathBuf::from("/wt/a"),
        refused: vec![],
        cap: Some(1048576),
    };
    assert_eq!(
        text(err),
        "/wt/a holds work that is not saved; the folder was not removed; next step: mortar save"
    );
}

#[test]
fn not_removed_unsaved_names_the_refused_files_and_the_cap() {
    let err = FolderError::NotRemovedUnsaved {
        path: PathBuf::from("/wt/a"),
        refused: vec![PathBuf::from("big.bin"), PathBuf::from("data/huge.tar")],
        cap: Some(1048576),
    };
    assert_eq!(
        text(err),
        "/wt/a holds files the snapshot size cap (1048576 bytes) refused: big.bin, data/huge.tar; they are in no \
         snapshot, so the folder was not removed; next step: move or delete them, then mortar save"
    );
}

#[test]
fn not_removed_unsaved_with_no_cap_leaves_the_cap_out() {
    let err = FolderError::NotRemovedUnsaved {
        path: PathBuf::from("/wt/a"),
        refused: vec![PathBuf::from("big.bin")],
        cap: None,
    };
    assert_eq!(
        text(err),
        "/wt/a holds files the snapshot size cap refused: big.bin; they are in no snapshot, so the folder was not \
         removed; next step: move or delete them, then mortar save"
    );
}

#[test]
fn not_joined_names_both_branches_and_the_next_step() {
    let err = FolderError::NotJoined {
        path: PathBuf::from("/wt/a"),
        branch: "refs/heads/req/folders/WO-1".to_string(),
        zone_branch: "refs/heads/zone/folders".to_string(),
    };
    assert_eq!(
        text(err),
        "/wt/a was not removed: branch refs/heads/req/folders/WO-1 is not contained in refs/heads/zone/folders; \
         next step: mortar join"
    );
}

#[test]
fn zone_folder_names_the_zone() {
    let err = FolderError::ZoneFolder {
        path: PathBuf::from("/wt/z"),
        zone: "folders".to_string(),
    };
    assert_eq!(
        text(err),
        "/wt/z is the zone folder for zone folders; remove takes request folders only; nothing changed"
    );
}

#[test]
fn ref_moved_names_the_ref_and_where_it_is() {
    let err = FolderError::RefMoved {
        refname: "refs/mortar/snap/folders/WO-1".to_string(),
        at: Some(Oid::parse("2222222222222222222222222222222222222222").unwrap()),
    };
    assert_eq!(
        text(err),
        "ref refs/mortar/snap/folders/WO-1 moved while it was being deleted (now at \
         2222222222222222222222222222222222222222); nothing was deleted; run again"
    );
}

#[test]
fn ref_moved_with_no_commit_says_so() {
    let err = FolderError::RefMoved {
        refname: "refs/heads/req/folders/WO-1".to_string(),
        at: None,
    };
    assert_eq!(
        text(err),
        "ref refs/heads/req/folders/WO-1 moved while it was being deleted (now at no commit); nothing was deleted; \
         run again"
    );
}

#[test]
fn attempt_refs_left_names_the_ref_where_it_is_and_the_removed_folders() {
    let err = FolderError::AttemptRefsLeft {
        moved: "refs/heads/try/folders/WO-1/2".to_string(),
        at: Some(Oid::parse("2222222222222222222222222222222222222222").unwrap()),
        removed: vec![PathBuf::from("/wt/try-1"), PathBuf::from("/wt/try-2")],
    };
    assert_eq!(
        text(err),
        "ref refs/heads/try/folders/WO-1/2 moved (now at 2222222222222222222222222222222222222222) before the attempt \
         branches were deleted; every attempt branch is kept; the attempt folders /wt/try-1, /wt/try-2 were already \
         removed; next step: run the same command again with the same saved attempts"
    );
}

#[test]
fn removed_refs_left_names_the_folder_both_refs_and_where_the_moved_one_is() {
    let err = FolderError::RemovedRefsLeft {
        path: PathBuf::from("/wt/a"),
        branch: "refs/heads/req/folders/WO-1".to_string(),
        moved: "refs/mortar/snap/folders/WO-1".to_string(),
        at: Some(Oid::parse("2222222222222222222222222222222222222222").unwrap()),
    };
    assert_eq!(
        text(err),
        "/wt/a was removed, but ref refs/mortar/snap/folders/WO-1 moved (now at \
         2222222222222222222222222222222222222222) before branch refs/heads/req/folders/WO-1 and its snapshot ref \
         were deleted; both are left as they are; the work they hold is in the zone; next step: mortar sweep"
    );
}

#[test]
fn removed_refs_left_with_no_commit_says_so() {
    let err = FolderError::RemovedRefsLeft {
        path: PathBuf::from("/wt/a"),
        branch: "refs/heads/req/folders/WO-1".to_string(),
        moved: "refs/heads/req/folders/WO-1".to_string(),
        at: None,
    };
    assert_eq!(
        text(err),
        "/wt/a was removed, but ref refs/heads/req/folders/WO-1 moved (now at no commit) before branch \
         refs/heads/req/folders/WO-1 and its snapshot ref were deleted; both are left as they are; the work they \
         hold is in the zone; next step: mortar sweep"
    );
}

#[test]
fn extra_ref_refused_names_the_ref_and_its_reason() {
    let err = FolderError::ExtraRefRefused {
        refname: "refs/heads/main".to_string(),
        reason: "is under refs/heads/".to_string(),
    };
    assert_eq!(
        text(err),
        "extra ref refs/heads/main is under refs/heads/; sweep_refs deletes no ref under refs/heads/, refs/mortar/ \
         or refs/remotes/, nor a protected one; nothing was swept"
    );
}

#[test]
fn extra_ref_refused_for_a_protected_ref() {
    let err = FolderError::ExtraRefRefused {
        refname: "refs/proven/folders".to_string(),
        reason: "is protected by land".to_string(),
    };
    assert_eq!(
        text(err),
        "extra ref refs/proven/folders is protected by land; sweep_refs deletes no ref under refs/heads/, \
         refs/mortar/ or refs/remotes/, nor a protected one; nothing was swept"
    );
}

#[test]
fn different_folder_names_an_attempt_folder_and_its_request() {
    let err = FolderError::DifferentFolder {
        path: PathBuf::from("/wt/a"),
        existing: existing(FolderKind::Attempt(2), Some("WO-1")),
    };
    assert_eq!(
        text(err),
        "/wt/a is already attempt 2's folder for zone folders, request WO-1; remove it or choose another path"
    );
}

#[test]
fn different_folder_names_an_attempt_folder_with_no_request() {
    let err = FolderError::DifferentFolder {
        path: PathBuf::from("/wt/a"),
        existing: existing(FolderKind::Attempt(7), None),
    };
    assert_eq!(
        text(err),
        "/wt/a is already attempt 7's folder for zone folders; remove it or choose another path"
    );
}

#[test]
fn request_branch_missing_names_the_branch_and_the_next_step() {
    let err = FolderError::RequestBranchMissing {
        branch: "req/folders/WO-1".to_string(),
    };
    assert_eq!(
        text(err),
        "request branch req/folders/WO-1 does not exist; make the request folder first"
    );
}

#[test]
fn attempt_missing_names_the_branch_and_what_keep_leaves() {
    let err = FolderError::AttemptMissing {
        branch: "try/folders/WO-1/3".to_string(),
    };
    assert_eq!(
        text(err),
        "no attempt branch try/folders/WO-1/3; nothing changed; if keep already ran, the request branch holds the \
         kept attempt; an attempt whose folder lost its branch cannot be kept: keep another attempt or drop them all"
    );
}

#[test]
fn attempts_unsaved_names_every_folder_and_the_next_step() {
    let err = FolderError::AttemptsUnsaved {
        paths: vec![PathBuf::from("/wt/try-1"), PathBuf::from("refs/heads/try/z/r/3")],
    };
    assert_eq!(
        text(err),
        "attempts /wt/try-1, refs/heads/try/z/r/3 are not marked saved; nothing changed; next step: mortar save in \
         each listed folder, then run the same command again; for a listed branch with no folder, make the attempt \
         folder again on its try branch and mortar save there, then run the same command again"
    );
}

#[test]
fn attempts_remain_names_the_folder_every_attempt_and_the_next_step() {
    let err = FolderError::AttemptsRemain {
        path: PathBuf::from("/wt/WO-1"),
        attempts: vec!["refs/heads/try/z/r/1".to_string(), "/wt/try-2".to_string()],
    };
    assert_eq!(
        text(err),
        "/wt/WO-1 was not removed: attempts of its request remain: refs/heads/try/z/r/1, /wt/try-2; nothing \
         changed; next step: keep one attempt or drop them all (drop_attempts); for an attempt folder that lost its \
         branch, remake_orphans first (mortar runs it), and with no snapshot of it, copy out anything wanted and \
         remove it by hand"
    );
}

#[test]
fn orphan_attempt_names_every_folder_and_remake_orphans() {
    let err = FolderError::OrphanAttempt {
        paths: vec![PathBuf::from("/wt/try-1"), PathBuf::from("/wt/try-2")],
    };
    assert_eq!(
        text(err),
        "attempt folders /wt/try-1, /wt/try-2 have lost their try branch; nothing changed; next step: remake_orphans \
         (mortar runs it before keep and drop), then run the same command again"
    );
}

#[test]
fn orphan_without_snapshot_names_every_folder_and_removal_by_hand() {
    let err = FolderError::OrphanWithoutSnapshot {
        paths: vec![PathBuf::from("/wt/try-1"), PathBuf::from("/wt/try-2")],
    };
    assert_eq!(
        text(err),
        "attempt folders /wt/try-1, /wt/try-2 have lost their try branch and have no snapshot, so Mortar holds no \
         copy of them; nothing changed; next step: copy out anything wanted, then remove each folder by hand"
    );
}

#[test]
fn request_moved_on_names_both_branches_and_the_next_step() {
    let err = FolderError::RequestMovedOn {
        branch: "req/folders/WO-1".to_string(),
        attempt: "try/folders/WO-1/2".to_string(),
    };
    assert_eq!(
        text(err),
        "request branch req/folders/WO-1 has commits attempt branch try/folders/WO-1/2 does not hold; keeping \
         try/folders/WO-1/2 would drop them; nothing changed; next step: bring them into try/folders/WO-1/2 or make \
         the attempts again"
    );
}

#[test]
fn attempt_folder_names_the_folder_and_its_number() {
    let err = FolderError::AttemptFolder {
        path: PathBuf::from("/wt/try-4"),
        k: 4,
    };
    assert_eq!(
        text(err),
        "/wt/try-4 is attempt 4's folder; remove takes request folders only; keep removes attempt folders; nothing \
         changed"
    );
}
