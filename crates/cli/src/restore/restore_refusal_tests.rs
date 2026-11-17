//! restore_refusal: an unsaved folder or a folder whose ref the restore deletes refuses with nothing moved; zone
//! folders are never checked (S-113(2), S-115(1)).

use std::path::PathBuf;

use folders::{FolderKind, FolderRecord};
use gitio::Oid;
use oplog::RefMove;

use super::{restore_refusal, FolderOnRef};
use crate::report::{ExitCode, Report};

const SAVE_NEXT: &str = "have the folder's agent run mr save, then repeat";
const DELETE_NEXT: &str =
    "restore without the refs those folders stand on, or have the harness remove those folders first, then repeat";

fn oid(c: char) -> Oid {
    Oid::parse(&c.to_string().repeat(40)).expect("a valid oid")
}

fn folder(path: &str, refname: &str, kind: FolderKind) -> FolderOnRef {
    let request = (kind != FolderKind::Zone).then(|| "fix-1".to_string());
    FolderOnRef {
        path: PathBuf::from(path),
        refname: refname.to_string(),
        record: FolderRecord {
            kind,
            zone: "core".to_string(),
            request,
            base: oid('a'),
            work_order_paths: Vec::new(),
            needed_paths: Vec::new(),
            parent: None,
        },
    }
}

fn mv(refname: &str, new: char) -> RefMove {
    RefMove { refname: refname.to_string(), old: Some(oid('b')), new: oid(new) }
}

fn refused(lines: &[&str], next: &str) -> Option<Report> {
    Some(Report {
        verb: "restore".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: lines.iter().map(ToString::to_string).collect(),
        owner: Some("the agent working in each folder named".to_string()),
        next: next.to_string(),
        fields: Vec::new(),
    })
}

#[test]
fn saved_folders_on_moved_refs_are_not_refused() {
    let folders = [folder("/w/a", "refs/heads/req/core/a", FolderKind::Request)];
    assert_eq!(restore_refusal(&folders, &[], &[mv("refs/heads/req/core/a", 'c')]), None);
}

#[test]
fn an_unsaved_folder_is_refused_with_save_as_next() {
    let folders = [folder("/w/a", "refs/heads/req/core/a", FolderKind::Request)];
    assert_eq!(
        restore_refusal(&folders, &[PathBuf::from("/w/a")], &[mv("refs/heads/req/core/a", 'c')]),
        refused(
            &["the folder /w/a on refs/heads/req/core/a holds work that is not saved", "nothing was moved"],
            SAVE_NEXT
        )
    );
}

#[test]
fn a_folder_whose_ref_is_deleted_is_refused_naming_it() {
    let folders = [folder("/w/t", "refs/heads/try/core/a/1", FolderKind::Attempt(1))];
    assert_eq!(
        restore_refusal(&folders, &[], &[mv("refs/heads/try/core/a/1", '0')]),
        refused(
            &["the folder /w/t stands on refs/heads/try/core/a/1, which the restore would delete", "nothing was moved"],
            DELETE_NEXT
        )
    );
}

#[test]
fn unsaved_and_deleted_folders_are_all_named_in_order_and_saving_comes_first() {
    let folders = [
        folder("/w/a", "refs/heads/req/core/a", FolderKind::Request),
        folder("/w/b", "refs/heads/req/core/b", FolderKind::Request),
    ];
    let moves = [mv("refs/heads/req/core/a", '0'), mv("refs/heads/req/core/b", 'c')];
    assert_eq!(
        restore_refusal(&folders, &[PathBuf::from("/w/b")], &moves),
        refused(
            &[
                "the folder /w/a stands on refs/heads/req/core/a, which the restore would delete",
                "the folder /w/b on refs/heads/req/core/b holds work that is not saved",
                "nothing was moved",
            ],
            SAVE_NEXT
        )
    );
}

#[test]
fn a_zone_folder_is_never_refused() {
    let folders = [folder("/w/z", "refs/heads/zone/core", FolderKind::Zone)];
    assert_eq!(restore_refusal(&folders, &[PathBuf::from("/w/z")], &[mv("refs/heads/zone/core", '0')]), None);
}

#[test]
fn a_delete_of_a_ref_no_folder_stands_on_is_not_refused() {
    let folders = [folder("/w/a", "refs/heads/req/core/a", FolderKind::Request)];
    assert_eq!(restore_refusal(&folders, &[], &[mv("refs/mortar/snap/core/a", '0')]), None);
}
