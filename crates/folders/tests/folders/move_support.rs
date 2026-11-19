//! Private fixtures for the move_folder ZITs; real sparse request folders and caller-held lock/recovery.
use crate::support::{run, setup, write, Fixture};
use folders::{make_request_folder, move_folder, FolderError};
use gitio::{Git, Oid};
use oplog::{with_lock, HeldLockError, KillPidProbe, LockName, OpPhase, OpRecord, OpResult, RefMove};
use std::{collections::BTreeMap, fs, path::{Path, PathBuf}};

pub const BRANCH: &str = "refs/heads/req/red/move";
pub const ACTOR: &str = "folder-move-integration-caller";

pub struct MoveFixture {
    pub f: Fixture,
    pub folder: PathBuf,
    pub local: Git,
    pub git_dir: PathBuf,
    pub new: Oid,
}

impl MoveFixture {
    pub fn new() -> Self {
        let f = Fixture::new();
        write(&f.repo.path, "kit/zone/file", b"moved new bytes\n");
        write(&f.repo.path, "kit/zone/new", b"new path bytes\n");
        fs::remove_file(f.repo.path.join("kit/zone/conflict")).unwrap();
        run(&f.git, &["add", "-A", "--", "kit/zone"]);
        run(&f.git, &["commit", "-m", "target folder tree"]);
        let new = f.git.rev_parse("HEAD").unwrap().unwrap();
        run(&f.git, &["update-ref", "refs/heads/zone/red", f.head.as_str()]);
        let folder = f.repo.path.join("moving-folder");
        make_request_folder(&setup(&f.git, &folder), "red", "move", &["kit/zone".into()]).unwrap();
        let local = Git { repo: folder.clone(), ..f.git.clone() };
        let git_dir = f.git.git_dir(&folder).unwrap();
        assert_eq!(fs::read(folder.join("kit/zone/file")).unwrap(), b"zone\n");
        assert_eq!(fs::read(folder.join("kit/zone/conflict")).unwrap(), b"base\n");
        assert!(!folder.join("kit/zone/new").exists());
        Self { f, folder, local, git_dir, new }
    }

    pub fn move_saved(&self, saved: bool) -> Result<(), HeldLockError<FolderError>> {
        caller_move(&self.f.git, &self.f.common, &self.folder, &self.f.head, &self.new, saved)
    }

    pub fn log(&self) -> Vec<u8> { fs::read(oplog::oplog_path(&self.f.common)).unwrap() }

    pub fn refs(&self) -> BTreeMap<String, String> {
        String::from_utf8(run(&self.f.git, &["for-each-ref", "--format=%(refname) %(objectname)"])).unwrap()
            .lines().map(|line| { let (name, id) = line.split_once(' ').unwrap(); (name.into(), id.into()) }).collect()
    }

    pub fn assert_moved(&self) {
        assert_eq!(self.f.git.read_ref(BRANCH).unwrap(), Some(self.new.clone()));
        assert_eq!(run(&self.local, &["write-tree"]), run(&self.f.git, &["rev-parse", &format!("{}^{{tree}}", self.new)]));
        assert_eq!(fs::read(self.folder.join("kit/zone/file")).unwrap(), b"moved new bytes\n");
        assert_eq!(fs::read(self.folder.join("kit/zone/new")).unwrap(), b"new path bytes\n");
        assert!(!self.folder.join("kit/zone/conflict").exists());
        assert!(run(&self.local, &["diff", "--name-only", "HEAD"]).is_empty());
        assert_eq!(fs::read(self.f.repo.path.join("kit/zone/file")).unwrap(), b"moved new bytes\n");
    }

    pub fn observe(&self) -> Observed {
        let mut files = Vec::new();
        read_files(&self.folder, &self.folder, &mut files);
        files.sort();
        Observed { refs: self.refs(), entries: run(&self.local, &["ls-files", "--stage", "-z"]), files,
            record: fs::read(self.git_dir.join("mortar-folder.json")).unwrap(), log: self.log() }
    }
}

pub fn caller_move(git: &Git, common: &Path, folder: &Path, old: &Oid, new: &Oid, saved: bool)
    -> Result<(), HeldLockError<FolderError>> {
    // The caller, not folders::move_folder, owns this precondition.
    with_lock(git, common, &LockName::Join("red".into()), "move-folder-caller", &KillPidProbe, || {
        oplog::recover(git, common).map_err(FolderError::Oplog)?;
        move_folder(git, folder, old, new, saved, ACTOR, None)
    })
}

pub fn assert_move_op(records: &[OpRecord], old: &Oid, new: &Oid) {
    assert_eq!(records.len(), 2, "one Intent/Done operation, not two moves");
    assert_eq!(records[0].phase, OpPhase::Intent);
    assert_eq!(records[0].result, None);
    assert_eq!(records[1].phase, OpPhase::Done);
    assert_eq!(records[1].result, Some(OpResult::Moved));
    assert!(!records[0].op_id.is_empty());
    assert_eq!(records[0].op_id, records[1].op_id);
    for record in records {
        assert_eq!(record.kind, "folder-move");
        assert_eq!(record.actor, ACTOR);
        assert_eq!(record.key, format!("folders:move:{BRANCH}@{old}..{new}"));
        assert_eq!(record.reason, format!("move folder branch {BRANCH} to {new}"));
        assert_eq!(record.moves, vec![RefMove { refname: BRANCH.into(), old: Some(old.clone()), new: new.clone() }]);
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Observed {
    refs: BTreeMap<String, String>,
    entries: Vec<u8>,
    files: Vec<(PathBuf, Vec<u8>)>,
    record: Vec<u8>,
    log: Vec<u8>,
}

fn read_files(root: &Path, dir: &Path, files: &mut Vec<(PathBuf, Vec<u8>)>) {
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if entry.file_type().unwrap().is_dir() { read_files(root, &path, files); }
        else { files.push((path.strip_prefix(root).unwrap().into(), fs::read(&path).unwrap())); }
    }
}
