//! #1179/#1193, SPEC-join WO-join-3/S50/S62/S64: public join/resolve/replay with real Git and caller-held locks.
use super::{author, join_refs, oid, Fixture, JOIN_ZONE, REQUEST_REF, ZONE_REF};
use gitio::{Git, Oid, RunOpts};
use join::{JoinAnswer, JoinError, JoinRunError};
use oplog::{with_lock, HeldLockError, KillPidProbe, LockName};
use std::{fs, path::PathBuf};

mod outcomes;
mod unusual_paths;
mod crashes;
mod folder_trees;

const ACTOR: &str = "resolve-integration-caller";
struct Paused { f: Fixture, path: String, base: Oid, bad: Oid, tip: Oid, zone: Oid, extras: Vec<Oid> }
impl Paused {
    fn new(kind: &str) -> Self { Self::named("conflict", kind) }
    fn named(path: &str, kind: &str) -> Self {
        let f = Fixture::new();
        let base_tree = f.tree(&[(path, b"base\n"), ("outside", b"old outside\n"), ("second", b"base second\n")]);
        let base = f.commit(&base_tree, &[], "base\n", &author("Base", "1700000000 +0000"));
        let bad_tree = f.tree(&[(path, b"request\n"), ("outside", b"old outside\n"), ("second", b"base second\n")]);
        let bad = f.commit(&bad_tree, std::slice::from_ref(&base), "conflicted message\n", &author("Zoë", "1700000100 -0530"));
        let mut tip = bad.clone(); let mut extras = Vec::new();
        if kind != "none" {
            let one_tree = f.tree(&[(path, b"request\n"), ("outside", b"old outside\n"), ("second", b"base second\n"), ("one", b"one\n")]);
            let one = f.commit(&one_tree, &[tip], "one message\n", &author("One", "1700000200 +0000"));
            let second: &[u8] = if kind == "conflict" { b"request second\n" } else { b"base second\n" };
            let two_tree = f.tree(&[(path, b"request\n"), ("outside", b"old outside\n"), ("second", second), ("one", b"one\n"), ("two", b"two\n")]);
            tip = f.commit(&two_tree, std::slice::from_ref(&one), "two message\n", &author("Two", "1700000300 +0000"));
            extras = vec![one, tip.clone()];
        }
        let second: &[u8] = if kind == "conflict" { b"zone second\n" } else { b"base second\n" };
        let zone_tree = f.tree(&[(path, b"zone\n"), ("outside", b"advanced zone work\n"), ("second", second)]);
        let zone = f.commit(&zone_tree, std::slice::from_ref(&base), "zone message\n", &author("Zone", "1700000400 +0000"));
        join_refs(&f, &zone, &tip);
        let r = Self { f, path: path.into(), base, bad, tip, zone, extras };
        let answer = caller(&r.f.git, &r.f, || join::join(&r.f.git, &r.f.repo.path, JOIN_ZONE, REQUEST_REF, ACTOR)).unwrap();
        assert!(matches!(answer, JoinAnswer::Conflicted { commit, ref files, .. } if commit == r.bad && files == &vec![PathBuf::from(path)]));
        r
    }
    fn state_path(&self) -> PathBuf { self.f.repo.path.join(".git/mortar-join.json") }
    fn state_text(&self) -> String { fs::read_to_string(self.state_path()).unwrap() }
    fn log(&self) -> Vec<u8> { fs::read(oplog::oplog_path(&self.f.repo.path.join(".git"))).unwrap_or_default() }
    fn resolve(&self, tree: &Oid) -> Result<JoinAnswer, JoinRunError> { self.resolve_with(&self.f.git, tree) }
    fn resolve_with(&self, git: &Git, tree: &Oid) -> Result<JoinAnswer, JoinRunError> {
        caller(git, &self.f, || join::resolve(git, &self.f.repo.path, tree, ACTOR))
    }
    fn caller_tree(&self) -> Oid {
        self.f.tree(&[(&self.path, b"resolved\n"), ("outside", b"unrelated caller edit\n"), ("second", b"caller second\n")])
    }
    fn first(&self) {
        let outside = fs::read(self.f.repo.path.join("outside")).unwrap();
        let outside_entry = self.f.run(&["ls-files", "--stage", "--", "outside"]);
        let gitio::MergeTreeOutcome::Conflicted { tree, .. } = self.f.git.merge_tree(&self.zone, Some(&self.base), &self.zone, &self.bad).unwrap() else { panic!("fixture must conflict") };
        let expected_bytes = self.f.blob(&tree, &self.path);
        let answer = self.resolve(&self.caller_tree()).unwrap();
        assert!(matches!(answer, JoinAnswer::FilesWritten { commit, files, skipped, gitlinks }
            if commit == self.bad && files == vec![PathBuf::from(&self.path)] && skipped.is_empty() && gitlinks.is_empty()));
        let bytes = fs::read(self.f.repo.path.join(&self.path)).unwrap();
        assert_eq!(bytes, expected_bytes, "write the actual conflicted tree version, not arbitrary markers");
        assert!(bytes.windows(7).any(|w| w == b"<<<<<<<"));
        assert_eq!(fs::read(self.f.repo.path.join("outside")).unwrap(), outside);
        assert_eq!(self.f.run(&["ls-files", "--stage", "--", "outside"]), outside_entry);
        assert!(self.state_text().contains("\"files_written\":true"));
    }
    fn commits(&self) -> Vec<u8> { self.f.run(&["cat-file", "--batch-all-objects", "--batch-check=%(objecttype) %(objectname)"]) }
    fn replace_state(&self, old: &str, new: &str) {
        let text = self.state_text(); assert_eq!(text.matches(old).count(), 1);
        fs::write(self.state_path(), text.replacen(old, new, 1)).unwrap(); // Exact field text, not a regex edit.
    }
}
fn caller<T>(git: &Git, f: &Fixture, work: impl FnOnce() -> Result<T, JoinRunError>) -> Result<T, JoinRunError> {
    let common = f.repo.path.join(".git");
    match with_lock(git, &common, &LockName::Join(JOIN_ZONE.into()), ACTOR, &KillPidProbe, || {
        oplog::recover(git, &common).map_err(JoinRunError::Oplog)?; work()
    }) {
        Ok(value) => Ok(value), Err(HeldLockError::Work(error)) => Err(error), Err(other) => panic!("caller lock: {other:?}"),
    }
}
fn rebased(answer: JoinAnswer, from: &Oid) -> Oid {
    match answer { JoinAnswer::Rebased { request_ref, from: actual, to, folder_tree: Some(_) } => { assert_eq!(request_ref, REQUEST_REF); assert_eq!(&actual, from); to }, other => panic!("expected resolved rebase with folder_tree: {other:?}") }
}
fn assert_one_rebase(r: &Paused, to: &Oid) {
    let records = oplog::read_op_log(&r.f.repo.path.join(".git")).unwrap().records;
    let records: Vec<_> = records.iter().filter(|r| r.kind == "join-rebase").collect();
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].phase, oplog::OpPhase::Intent); assert_eq!(records[1].phase, oplog::OpPhase::Done);
    for record in records {
        assert_eq!(record.actor, ACTOR);
        assert_eq!(record.moves, vec![oplog::RefMove { refname: REQUEST_REF.into(), old: Some(r.tip.clone()), new: to.clone() }]);
    }
    assert_eq!(r.f.git.read_ref(REQUEST_REF).unwrap(), Some(to.clone()));
    assert_eq!(r.f.git.read_ref(ZONE_REF).unwrap(), Some(r.zone.clone()));
    assert!(!r.state_path().exists());
}
