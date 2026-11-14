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
