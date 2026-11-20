//! remake_orphans in a throwaway repository: the cases src/attempts/remake_orphans.rs's header names.
//! Each orphan is made by writing its trysnap commit (attempt_support's trysnap), some after committing in the
//! attempt folder, then deleting its try branch by hand; two of the refusal test's orphans instead have no trysnap
//! ref, or one set by update-ref to the fixture's parentless head.
use super::attempt_support::{oid, trysnap_ref, try_ref, Attempts, REQ_REF};
use super::maintenance_support::{records, ACTOR};
use crate::support::{run, write};
use folders::FolderError;
use gitio::Oid;
use oplog::{OpPhase, OpResult, RefMove};
use std::{fs, path::Path};

const CHANGE_ID: &str = "kkkkllllmmmmnnnnooooppppqqqqrrrr";

/// Commits one new file in attempt k's folder with `message`; returns the commit.
fn commit_with(a: &Attempts, k: u32, message: &str) -> Oid {
    let local = a.local(k);
    write(&a.folder(k), &format!("kit/zone/attempt-{k}"), b"attempt work\n");
    run(&local, &["add", "kit/zone"]);
    run(&local, &["commit", "-q", "-m", message]);
    oid(run(&local, &["rev-parse", "HEAD"]))
}

/// The tree a commit holds.
fn tree_of(a: &Attempts, commit: &Oid) -> Oid { oid(run(&a.f.git, &["rev-parse", &format!("{commit}^{{tree}}")])) }

/// What git log prints for `commit` with `format`, trailing newlines dropped.
fn show(a: &Attempts, commit: &Oid, format: &str) -> String {
    let bytes = run(&a.f.git, &["log", "-1", "--date=raw", &format!("--format={format}"), commit.as_str()]);
    String::from_utf8(bytes).unwrap().trim_end().to_string()
}

/// Every file under `dir` with its bytes, sorted by path.
fn files(dir: &Path) -> Vec<(String, Vec<u8>)> {
    let mut out = Vec::new();
    for item in fs::read_dir(dir).unwrap() {
        let path = item.unwrap().path();
        if path.is_dir() { out.extend(files(&path)); } else { out.push((path.display().to_string(), fs::read(&path).unwrap())); }
    }
    out.sort();
    out
}

/// The folder's index as git lists it.
fn index(a: &Attempts, k: u32) -> Vec<u8> { run(&a.local(k), &["ls-files", "--stage", "-z"]) }

/// The one folder-branch op from `start` that made try branch k at `tip`.
fn assert_remade_op(a: &Attempts, start: usize, k: u32, tip: &Oid) {
    let ops = records(&a.f);
    let ops = &ops[start..];
    assert_eq!(ops.len(), 2);
    assert_eq!(ops[0].phase, OpPhase::Intent);
    assert_eq!(ops[0].result, None);
    assert_eq!(ops[1].phase, OpPhase::Done);
    assert_eq!(ops[1].result, Some(OpResult::Moved));
    assert_eq!(ops[0].op_id, ops[1].op_id);
    for op in ops {
        assert_eq!(op.kind, "folder-branch");
        assert_eq!(op.actor, ACTOR);
        assert_eq!(op.moves, vec![RefMove { refname: try_ref(k), old: None, new: tip.clone() }]);
    }
}

#[test]
fn an_orphan_with_a_trysnap_is_remade_at_a_new_commit_with_its_tree_and_its_parents_trailers_files_untouched_in_one_op() {
    let a = Attempts::with(1);
    let message = format!("attempt work\n\nMortar-Change-Id: {CHANGE_ID}\nMortar-Request: one\nMortar-Zone: red");
    let parent = commit_with(&a, 1, &message);
    let snap = a.trysnap(1, true);
    a.orphan(1);
    let tree = tree_of(&a, &snap);
    assert_ne!(tree, tree_of(&a, &parent), "the snapshot was saved after a change");
    let before = files(&a.folder(1));
    let start = records(&a.f).len();
    assert_eq!(a.remake().unwrap(), vec![a.folder(1)]);
    let remade = a.f.git.read_ref(&try_ref(1)).unwrap().unwrap();
    assert_ne!(remade, snap);
    assert_eq!(tree_of(&a, &remade), tree);
    assert_eq!(show(&a, &remade, "%P"), parent.to_string());
    assert_eq!(show(&a, &remade, "%B"), format!(
        "one: attempt 1 re-made from its snapshot\n\nMortar-Change-Id: {CHANGE_ID}\nMortar-Request: one\nMortar-Zone: red"
    ));
    let who = "Snapshot Saver|saver@example.invalid|1700000000 +0000";
    assert_eq!(show(&a, &remade, "%an|%ae|%ad|%cn|%ce|%cd"), format!("{who}|{who}"));
    assert_eq!(files(&a.folder(1)), before);
    assert_eq!(oid(run(&a.local(1), &["write-tree"])), tree);
    assert_eq!(run(&a.local(1), &["status", "--porcelain"]), b"");
    assert_eq!(a.f.git.read_ref(&trysnap_ref(1)).unwrap(), Some(snap.clone()));
    assert_remade_op(&a, start, 1, &remade);
}

#[test]
fn a_parent_without_this_requests_trailers_gives_the_remade_commit_none() {
    let a = Attempts::with(2);
    commit_with(&a, 1, "attempt work");
    commit_with(&a, 2, &format!("attempt work\n\nMortar-Change-Id: {CHANGE_ID}\nMortar-Request: two\nMortar-Zone: red"));
    a.trysnap(1, true);
    a.trysnap(2, false);
    a.orphan(1);
    a.orphan(2);
    assert_eq!(a.remake().unwrap(), vec![a.folder(1), a.folder(2)]);
    for k in 1..=2 {
        let remade = a.f.git.read_ref(&try_ref(k)).unwrap().unwrap();
        assert_eq!(show(&a, &remade, "%B"), format!("one: attempt {k} re-made from its snapshot"), "attempt {k}");
    }
}

#[test]
fn a_remade_attempt_is_kept_like_any_other_and_the_request_branch_holds_no_snapshot_commit() {
    let a = Attempts::with(2);
    let head = a.f.head.clone();
    let parent = commit_with(&a, 1, "attempt work");
    let snap = a.trysnap(1, true);
    a.orphan(1);
    a.remake().unwrap();
    let remade = a.f.git.read_ref(&try_ref(1)).unwrap().unwrap();
    assert_eq!(a.keep(&a.f.git, 1, &[(2, true)]).unwrap(), (head.clone(), remade.clone()));
    assert_eq!(a.f.git.read_ref(REQ_REF).unwrap(), Some(remade.clone()));
    let between = run(&a.f.git, &["rev-list", "--topo-order", &format!("{head}..{remade}")]);
    assert_eq!(String::from_utf8(between).unwrap(), format!("{remade}\n{parent}\n"));
    assert_ne!(remade, snap);
}

#[test]
fn a_repeat_after_completion_returns_empty_and_logs_nothing() {
    let a = Attempts::with(1);
    a.trysnap(1, true);
    a.orphan(1);
    assert_eq!(a.remake().unwrap(), vec![a.folder(1)]);
    let before = a.state();
    let index_before = index(&a, 1);
    assert_eq!(a.remake().unwrap(), Vec::<std::path::PathBuf>::new());
    assert_eq!(a.state(), before);
    assert_eq!(index(&a, 1), index_before);
}

#[test]
fn a_repeat_after_a_kill_before_the_op_remakes_it() {
    let a = Attempts::with(1);
    let snap = a.trysnap(1, true);
    a.orphan(1);
    let tree = tree_of(&a, &snap);
    a.f.git.set_index(&a.folder(1), &tree).unwrap();
    let start = records(&a.f).len();
    assert_eq!(a.remake().unwrap(), vec![a.folder(1)]);
    let remade = a.f.git.read_ref(&try_ref(1)).unwrap().unwrap();
    assert_eq!(tree_of(&a, &remade), tree);
    assert_eq!(show(&a, &remade, "%P"), a.f.head.to_string());
    assert_eq!(oid(run(&a.local(1), &["write-tree"])), tree);
    assert_remade_op(&a, start, 1, &remade);
}

#[test]
fn an_orphan_with_no_trysnap_or_a_parentless_one_is_orphan_without_snapshot_naming_them_with_nothing_changed() {
    let a = Attempts::with(3);
    a.trysnap(3, true);
    run(&a.f.git, &["update-ref", &trysnap_ref(2), a.f.head.as_str()]);
    for k in 1..=3 { a.orphan(k); }
    let before = a.state();
    let indexes: Vec<Vec<u8>> = (1..=3).map(|k| index(&a, k)).collect();
    match a.remake() {
        Err(FolderError::OrphanWithoutSnapshot { paths }) => assert_eq!(paths, vec![a.folder(1), a.folder(2)]),
        other => panic!("expected OrphanWithoutSnapshot, got {other:?}"),
    }
    assert_eq!(a.state(), before);
    assert_eq!((1..=3).map(|k| index(&a, k)).collect::<Vec<_>>(), indexes);
}

#[test]
fn no_orphans_returns_empty_with_nothing_changed() {
    let a = Attempts::with(2);
    a.trysnap(1, true);
    let before = a.state();
    assert_eq!(a.remake().unwrap(), Vec::<std::path::PathBuf>::new());
    assert_eq!(a.state(), before);
}
