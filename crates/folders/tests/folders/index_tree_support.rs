//! Fixture for the ZITs of move_folder with an index tree: a request folder at the old tip
//! O whose resolved path holds the resolution bytes, the index tree t (O's tree with the resolution: the content
//! gitio's tree_with_paths gives join, here taken from a commit's tree) and the new tip N, which changes the zone
//! path, keeps the resolution and adds a path. The caller runs move_folder with Some(t) under the caller's lock after
//! oplog::recover, as caller_move does.
use super::move_support::{ACTOR, BRANCH};
use crate::support::{run, setup, write, Fixture};
use folders::{make_request_folder, move_folder, FolderError};
use gitio::{Git, Oid};
use oplog::{with_lock, HeldLockError, KillPidProbe, LockName};
use std::{fs, path::PathBuf};

/// Changed by the zone between O and N.
pub const ZONE: &str = "kit/zone/file";
/// The path the resolve settled; t and N both hold the resolution.
pub const RESOLVED: &str = "kit/zone/conflict";
/// Neither the zone nor the resolve changes it.
pub const KEEP: &str = "kit/zone/keep";
/// Only N has it.
pub const ADDED: &str = "kit/zone/added";

pub struct TreeFixture {
    pub f: Fixture,
    pub folder: PathBuf,
    pub local: Git,
    pub old: Oid,
    pub tree: Oid,
    pub new: Oid,
}

impl TreeFixture {
    pub fn new() -> Self {
        let f = Fixture::new();
        write(&f.repo.path, KEEP, b"keep\n");
        let old = commit(&f, "old request tip");
        write(&f.repo.path, RESOLVED, b"resolution bytes\n");
        let resolved = commit(&f, "old tip with the resolution");
        write(&f.repo.path, ZONE, b"zone change at new\n");
        write(&f.repo.path, ADDED, b"added at new\n");
        let new = commit(&f, "new request tip");
        let tree = run(&f.git, &["rev-parse", &format!("{resolved}^{{tree}}")]);
        let tree = Oid::parse(std::str::from_utf8(&tree).unwrap().trim()).unwrap();
        run(&f.git, &["update-ref", "refs/heads/zone/red", old.as_str()]);
        let folder = f.repo.path.join("resolved-folder");
        make_request_folder(&setup(&f.git, &folder), "red", "move", &["kit/zone".into()]).unwrap();
        let local = Git { repo: folder.clone(), ..f.git.clone() };
        assert_eq!(f.git.read_ref(BRANCH).unwrap(), Some(old.clone()));
        assert_eq!(fs::read(folder.join(RESOLVED)).unwrap(), b"base\n");
        write(&folder, RESOLVED, b"resolution bytes\n");
        Self { f, folder, local, old, tree, new }
    }

    /// The request branch put at N, as join's own op does before the caller moves the folder.
    pub fn branch_at_new(&self) {
        run(&self.f.git, &["update-ref", BRANCH, self.new.as_str(), self.old.as_str()]);
    }

    pub fn move_with_tree(&self) -> Result<(), HeldLockError<FolderError>> {
        let (git, common) = (&self.f.git, &self.f.common);
        with_lock(git, common, &LockName::Join("red".into()), "move-folder-caller", &KillPidProbe, || {
            oplog::recover(git, common).map_err(FolderError::Oplog)?;
            move_folder(git, &self.folder, &self.old, &self.new, true, ACTOR, Some(&self.tree))
        })
    }

    /// The folder's index as a tree id, with git's trailing newline.
    pub fn index_tree(&self) -> Vec<u8> { run(&self.local, &["write-tree"]) }

    /// A commit's or tree's tree id, with git's trailing newline.
    pub fn tree_of(&self, id: &Oid) -> Vec<u8> { run(&self.f.git, &["rev-parse", &format!("{id}^{{tree}}")]) }

    pub fn read(&self, path: &str) -> Vec<u8> { fs::read(self.folder.join(path)).unwrap() }

    pub fn status(&self, path: &str) -> Vec<u8> { run(&self.local, &["status", "--porcelain", "--", path]) }

    pub fn log(&self) -> Vec<u8> { fs::read(oplog::oplog_path(&self.f.common)).unwrap() }

    pub fn branch(&self) -> Option<Oid> { self.f.git.read_ref(BRANCH).unwrap() }
}

fn commit(f: &Fixture, message: &str) -> Oid {
    run(&f.git, &["add", "-A", "--", "kit/zone"]);
    run(&f.git, &["commit", "-m", message]);
    f.git.rev_parse("HEAD").unwrap().unwrap()
}
