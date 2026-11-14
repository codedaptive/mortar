//! remake_plan: one Remake per orphan in k order at its snapshot's tree and first parent; every orphan with no
//! snapshot, a None one or a parentless one refused in k order and nothing planned; no orphans plan nothing.

use std::path::PathBuf;

use gitio::Oid;

use super::{remake_plan, Remake, Snap};
use crate::make::FolderError;

const T1: &str = "1111111111111111111111111111111111111111";
const P1: &str = "2222222222222222222222222222222222222222";
const T2: &str = "3333333333333333333333333333333333333333";
const P2: &str = "4444444444444444444444444444444444444444";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn orphan(k: u32) -> (u32, PathBuf) {
    (k, PathBuf::from(format!("/t/try-{k}")))
}

fn snap(k: u32, tree: &str, parent: Option<&str>) -> (u32, Option<Snap>) {
    (k, Some(Snap { tree: oid(tree), first_parent: parent.map(oid) }))
}

#[test]
fn no_orphans_plan_nothing() {
    assert_eq!(remake_plan(&[], &[snap(1, T1, Some(P1))]).unwrap(), Vec::<Remake>::new());
}

#[test]
fn each_orphan_is_remade_at_its_own_snapshot_in_k_order() {
    let planned = remake_plan(&[orphan(2), orphan(1)], &[snap(1, T1, Some(P1)), snap(2, T2, Some(P2))]).unwrap();
    assert_eq!(
        planned,
        vec![
            Remake { k: 1, folder: PathBuf::from("/t/try-1"), tree: oid(T1), parent: oid(P1) },
            Remake { k: 2, folder: PathBuf::from("/t/try-2"), tree: oid(T2), parent: oid(P2) },
        ]
    );
}

#[test]
fn an_orphan_with_no_snap_entry_is_refused_naming_its_folder() {
    match remake_plan(&[orphan(1), orphan(2)], &[snap(1, T1, Some(P1))]) {
        Err(FolderError::OrphanWithoutSnapshot { paths }) => assert_eq!(paths, vec![PathBuf::from("/t/try-2")]),
        other => panic!("expected OrphanWithoutSnapshot, got {other:?}"),
    }
}

#[test]
fn an_orphan_whose_snap_is_none_is_refused() {
    match remake_plan(&[orphan(1)], &[(1, None)]) {
        Err(FolderError::OrphanWithoutSnapshot { paths }) => assert_eq!(paths, vec![PathBuf::from("/t/try-1")]),
        other => panic!("expected OrphanWithoutSnapshot, got {other:?}"),
    }
}

#[test]
fn an_orphan_whose_snapshot_has_no_parent_is_refused() {
    match remake_plan(&[orphan(1)], &[snap(1, T1, None)]) {
        Err(FolderError::OrphanWithoutSnapshot { paths }) => assert_eq!(paths, vec![PathBuf::from("/t/try-1")]),
        other => panic!("expected OrphanWithoutSnapshot, got {other:?}"),
    }
}

#[test]
fn every_orphan_without_a_snapshot_is_named_in_k_order_and_nothing_is_planned() {
    let orphans = [orphan(4), orphan(2), orphan(3), orphan(1)];
    let snaps = [snap(2, T2, Some(P2)), (3, None), snap(4, T1, None)];
    match remake_plan(&orphans, &snaps) {
        Err(FolderError::OrphanWithoutSnapshot { paths }) => assert_eq!(
            paths,
            vec![PathBuf::from("/t/try-1"), PathBuf::from("/t/try-3"), PathBuf::from("/t/try-4")]
        ),
        other => panic!("expected OrphanWithoutSnapshot, got {other:?}"),
    }
}

#[test]
fn a_snap_for_another_k_does_not_serve_this_orphan() {
    match remake_plan(&[orphan(1)], &[snap(2, T2, Some(P2))]) {
        Err(FolderError::OrphanWithoutSnapshot { paths }) => assert_eq!(paths, vec![PathBuf::from("/t/try-1")]),
        other => panic!("expected OrphanWithoutSnapshot, got {other:?}"),
    }
}
