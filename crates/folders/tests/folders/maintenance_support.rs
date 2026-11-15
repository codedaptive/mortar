//! Private real-repository fixtures for WO-orc-folders-it-4; caller holds lock and recovers.
use crate::support::{run, setup, Fixture};
use folders::{make_request_folder, FolderError};
use gitio::{Git, Oid};
use oplog::{with_lock, HeldLockError, KillPidProbe, LockName, OpPhase, OpRecord, OpResult, ProtectedRef, RefMove};
use std::{collections::BTreeMap, fs, path::{Path, PathBuf}};
use testkit::write_git_shim;
pub const ACTOR: &str = "folders-maintenance-integration-caller";
pub const BRANCH: &str = "refs/heads/req/red/one";
pub const SNAP: &str = "refs/mortar/snap/red/one";

pub struct Removal { pub f: Fixture, pub folder: PathBuf, pub local: Git, pub git_dir: PathBuf }
impl Removal {
    pub fn new() -> Self {
        let f = Fixture::new();
        run(&f.git, &["update-ref", "refs/heads/zone/red", f.head.as_str()]);
        let folder = f.repo.path.join("removable");
        make_request_folder(&setup(&f.git, &folder), "red", "one", &["kit/zone".into()]).unwrap();
        let local = Git { repo: folder.clone(), ..f.git.clone() };
        let git_dir = f.git.git_dir(&folder).unwrap();
        Self { f, folder, local, git_dir }
    }
    pub fn remove(&self, git: &Git) -> Result<(), FolderError> {
        caller(git, &self.f.common, || folders::remove(git, &self.folder, true, &[], None, ACTOR))
    }
    pub fn snapshot(&self) { run(&self.f.git, &["update-ref", SNAP, self.f.head.as_str()]); }
}

pub fn caller<T>(git: &Git, common: &Path, work: impl FnOnce() -> Result<T, FolderError>) -> Result<T, FolderError> {
    match with_lock(git, common, &LockName::Folders, "maintenance-integration", &KillPidProbe, || {
        oplog::recover(git, common).map_err(FolderError::Oplog)?;
        work()
    }) {
        Ok(value) => Ok(value),
        Err(HeldLockError::Work(error)) => Err(error),
        Err(other) => panic!("unexpected caller-lock failure: {other:?}"),
    }
}
pub fn sweep(f: &Fixture, git: &Git, zones: &[String], extra: &[String], protected: &[ProtectedRef]) -> Result<Vec<String>, FolderError> {
    caller(git, &f.common, || folders::sweep_refs(git, ACTOR, zones, extra, protected))
}
pub fn child(f: &Fixture, message: &str) -> Oid {
    let tree = String::from_utf8(run(&f.git, &["rev-parse", "HEAD^{tree}"])).unwrap();
    let bytes = run(&f.git, &["commit-tree", tree.trim_end(), "-p", f.head.as_str(), "-m", message]);
    Oid::parse(std::str::from_utf8(&bytes).unwrap().trim_end()).unwrap()
}
pub fn refs(git: &Git) -> BTreeMap<String, String> {
    String::from_utf8(run(git, &["for-each-ref", "--format=%(refname) %(objectname)"])).unwrap().lines()
        .map(|line| { let (name, id) = line.split_once(' ').unwrap(); (name.into(), id.into()) }).collect()
}
pub fn log(f: &Fixture) -> Vec<u8> { fs::read(oplog::oplog_path(&f.common)).unwrap() }
pub fn records(f: &Fixture) -> Vec<OpRecord> { oplog::read_op_log(&f.common).unwrap().records }

pub fn assert_delete_pair(records: &[OpRecord], kind: &str, names: &[&str], old: &Oid) {
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].phase, OpPhase::Intent);
    assert_eq!(records[0].result, None);
    assert_eq!(records[1].phase, OpPhase::Done);
    assert_eq!(records[1].result, Some(OpResult::Moved));
    assert!(!records[0].op_id.is_empty());
    assert_eq!(records[0].op_id, records[1].op_id);
    let wanted = names.iter().map(|name| RefMove { refname: (*name).into(), old: Some(old.clone()), new: Oid::parse(&"0".repeat(40)).unwrap() }).collect::<Vec<_>>();
    for record in records {
        assert_eq!(record.kind, kind);
        assert_eq!(record.actor, ACTOR);
        assert_eq!(record.moves, wanted);
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Observed { refs: BTreeMap<String, String>, index: Vec<u8>, files: Vec<(PathBuf, Option<Vec<u8>>)>, record: Option<Vec<u8>>, log: Vec<u8> }
pub fn observe(m: &Removal) -> Observed {
    let mut files = Vec::new(); walk(&m.folder, &m.folder, &mut files); files.sort();
    Observed { refs: refs(&m.f.git), index: run(&m.local, &["ls-files", "--stage", "-z"]), files,
        record: fs::read(m.git_dir.join("mortar-folder.json")).ok(), log: log(&m.f) }
}
fn walk(root: &Path, dir: &Path, out: &mut Vec<(PathBuf, Option<Vec<u8>>)>) {
    for item in fs::read_dir(dir).unwrap() {
        let item = item.unwrap(); let path = item.path(); let relative = path.strip_prefix(root).unwrap().into();
        if item.file_type().unwrap().is_dir() { out.push((relative, None)); walk(root, &path, out); }
        else { out.push((relative, Some(fs::read(path).unwrap()))); }
    }
}

/// Observe real maintenance; optional boundary action happens only after real worktree removal or at the CAS.
pub fn instrument(f: &Fixture, action: &str, name: &str, new: &Oid) -> (Git, PathBuf, PathBuf) {
    let shim = f.common.join(format!("maintenance-{action}.sh"));
    let marker = f.common.join(format!("maintenance-{action}.after-remove"));
    let maintenance = f.common.join(format!("maintenance-{action}.maintenance"));
    write_git_shim(&shim, br#"#!/bin/sh
set -eu
if [ "${3-}" = maintenance ]; then printf 'maintenance\n' >> "$MORTAR_TEST_MAINTENANCE"; fi
if [ "$MORTAR_TEST_ACTION" = git-error ] && [ -f "$MORTAR_TEST_AFTER_REMOVE" ]; then
  case " $* " in *' --git-common-dir '*) printf 'fixture common-dir error\n' >&2; exit 67;; esac
fi
if [ "$MORTAR_TEST_ACTION" = sweep-race ] && [ "${3-}" = update-ref ] && [ "${4-}" = --stdin ] && [ ! -f "$MORTAR_TEST_AFTER_REMOVE" ]; then
  git -C "$2" update-ref "$MORTAR_TEST_REF" "$MORTAR_TEST_NEW"
  printf 'raced\n' > "$MORTAR_TEST_AFTER_REMOVE"
fi
if [ "${3-}" = worktree ] && [ "${4-}" = remove ]; then
  git "$@"
  printf 'removed\n' > "$MORTAR_TEST_AFTER_REMOVE"
  case "$MORTAR_TEST_ACTION" in
    move-ref) git -C "$2" update-ref "$MORTAR_TEST_REF" "$MORTAR_TEST_NEW";;
    oplog-error) printf 'not-json\n' >> "$MORTAR_TEST_OPLOG";;
  esac
  exit 0
fi
exec git "$@"
"#).unwrap();
    let mut git = f.git.clone(); git.program = shim;
    git.env.extend([
        ("MORTAR_TEST_ACTION".into(), action.into()), ("MORTAR_TEST_REF".into(), name.into()),
        ("MORTAR_TEST_NEW".into(), new.as_str().into()), ("MORTAR_TEST_OPLOG".into(), oplog::oplog_path(&f.common).into_os_string()),
        ("MORTAR_TEST_AFTER_REMOVE".into(), marker.clone().into_os_string()), ("MORTAR_TEST_MAINTENANCE".into(), maintenance.clone().into_os_string()),
    ]);
    (git, marker, maintenance)
}
