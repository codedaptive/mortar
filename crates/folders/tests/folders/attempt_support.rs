//! Private real-repository fixtures for the attempt zone integration tests (WO-folders-6c, WO-folders-6d): zone branch
//! red, request folder one on req/red/one, its attempt folders made by make_attempts, and their trysnap commits, in a
//! disposable TestRepo.
use super::maintenance_support::{caller, log, refs, ACTOR};
use crate::support::{run, setup, write, Fixture};
use folders::plan::attempt_path;
use folders::{make_attempts, make_request_folder, FolderError, FolderRecord};
use gitio::{Git, Oid, Person};
use std::{collections::BTreeMap, fs, path::{Path, PathBuf}};

pub const REQ_REF: &str = "refs/heads/req/red/one";

pub fn try_ref(k: u32) -> String { format!("refs/heads/try/red/one/{k}") }
pub fn trysnap_ref(k: u32) -> String { format!("refs/mortar/trysnap/red/one/{k}") }

/// The author of every trysnap commit trysnap writes; its own name and date, so a commit that takes it over as author
/// and committer is told apart from one the fixture's identity made.
fn saver() -> Person {
    Person { name: "Snapshot Saver".into(), email: "saver@example.invalid".into(), date: Some("1700000000 +0000".into()) }
}

pub fn oid(bytes: Vec<u8>) -> Oid { Oid::parse(std::str::from_utf8(&bytes).unwrap().trim_end()).unwrap() }

/// make_attempts for zone red, request one, cone kit/zone, attempts under `dir`; it takes the folders lock itself.
pub fn make(f: &Fixture, dir: &Path, n: u32) -> Result<Vec<FolderRecord>, FolderError> {
    make_attempts(&setup(&f.git, dir), "red", "one", &["kit/zone".into()], n)
}

/// A repository with zone branch red at the fixture head and request folder one cut from it; paths are canonical, as
/// git's worktree list prints them.
pub struct Attempts { pub f: Fixture, pub request: PathBuf, pub dir: PathBuf }
impl Attempts {
    pub fn new() -> Self {
        let f = Fixture::new();
        run(&f.git, &["update-ref", "refs/heads/zone/red", f.head.as_str()]);
        let root = fs::canonicalize(&f.repo.path).unwrap();
        let request = root.join("request");
        make_request_folder(&setup(&f.git, &request), "red", "one", &["kit/zone".into()]).unwrap();
        Self { f, request, dir: root.join("attempts") }
    }
    /// new() with attempts 1..=n made.
    pub fn with(n: u32) -> Self { let a = Self::new(); a.make(n).unwrap(); a }
    pub fn make(&self, n: u32) -> Result<Vec<FolderRecord>, FolderError> { make(&self.f, &self.dir, n) }
    pub fn folder(&self, k: u32) -> PathBuf { attempt_path(&self.dir, k) }
    pub fn local(&self, k: u32) -> Git { Git { repo: self.folder(k), ..self.f.git.clone() } }
    /// Commits one new file in attempt k's folder, so its try branch moves past the request tip; returns the commit.
    pub fn commit_in(&self, k: u32) -> Oid {
        let local = self.local(k);
        write(&self.folder(k), &format!("kit/zone/attempt-{k}"), b"attempt work\n");
        run(&local, &["add", "kit/zone"]);
        run(&local, &["commit", "-m", "attempt work"]);
        local.rev_parse("HEAD").unwrap().unwrap()
    }
    /// Deletes attempt k's try branch by hand, leaving its folder: an orphan attempt folder.
    pub fn orphan(&self, k: u32) { run(&self.f.git, &["update-ref", "-d", &try_ref(k)]); }
    /// Points every attempt's trysnap ref at the fixture head, as a save in each would.
    pub fn trysnaps(&self, n: u32) {
        for k in 1..=n { run(&self.f.git, &["update-ref", &trysnap_ref(k), self.f.head.as_str()]); }
    }
    /// Writes attempt k's trysnap commit as a save would (SPEC-snapshot WO-snapshot-1) and points its trysnap ref at
    /// it: parents [try tip, the previous trysnap commit when the ref is set], author saver(). With `change` a new file
    /// kit/zone/saved-<k> is written in the folder first, so the commit's tree is the folder's files and differs from
    /// the try tip's tree; without it, the tree is the try tip's. The folder's index is left at the try tip's tree.
    /// Run it while the try branch exists; returns the commit.
    pub fn trysnap(&self, k: u32, change: bool) -> Oid {
        let local = self.local(k);
        if change {
            write(&self.folder(k), &format!("kit/zone/saved-{k}"), b"saved after a change\n");
            run(&local, &["add", "kit/zone"]);
        }
        let tree = oid(run(&local, &["write-tree"]));
        run(&local, &["reset", "-q"]);
        let mut parents = vec![self.f.git.read_ref(&try_ref(k)).unwrap().unwrap()];
        parents.extend(self.f.git.read_ref(&trysnap_ref(k)).unwrap());
        let snap = self.f.git.commit_tree(&tree, &parents, "snapshot", Some(&saver()), Some(&saver())).unwrap();
        run(&self.f.git, &["update-ref", &trysnap_ref(k), snap.as_str()]);
        snap
    }
    /// remake_orphans under the caller's lock after recover, as keep and drop_attempts are called.
    pub fn remake(&self) -> Result<Vec<PathBuf>, FolderError> {
        caller(&self.f.git, &self.f.common, || folders::remake_orphans(&self.f.git, "red", "one", ACTOR))
    }
    /// keep under the caller's lock after recover, through `git`.
    pub fn keep(&self, git: &Git, k: u32, saved: &[(u32, bool)]) -> Result<(Oid, Oid), FolderError> {
        caller(git, &self.f.common, || folders::keep(git, "red", "one", k, saved, ACTOR))
    }
    /// remove of the request folder, saved, under the caller's lock.
    pub fn remove_request(&self) -> Result<(), FolderError> {
        caller(&self.f.git, &self.f.common, || folders::remove(&self.f.git, &self.request, true, &[], None, ACTOR))
    }
    pub fn state(&self) -> State {
        State { refs: refs(&self.f.git), worktrees: worktrees(&self.f.git), log: log(&self.f) }
    }
}

/// What a refusal must leave untouched: every ref, every worktree (path and branch), and the op log's bytes.
#[derive(Debug, PartialEq, Eq)]
pub struct State { pub refs: BTreeMap<String, String>, pub worktrees: Vec<(PathBuf, Option<String>)>, pub log: Vec<u8> }

pub fn worktrees(git: &Git) -> Vec<(PathBuf, Option<String>)> {
    git.worktree_list().unwrap().into_iter().map(|w| (w.path, w.branch)).collect()
}
