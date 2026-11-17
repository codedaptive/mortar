//! Disposable real-object setup; reuses cli_rit's isolated process/Git fixture rather than a fake kit boundary.
use crate::{json::{check, Report}, support::{raw, Fixture, ZONE}};
use gitio::{Git, Oid};
use land::{LandAnswer, LandRefs, Squash, SquashAnswer};
use oplog::{KillPidProbe, LockName};
use std::{fs, path::{Path, PathBuf}, process::Command};

pub struct Harness { pub f: Fixture, pub bare: PathBuf }
impl Harness {
    pub fn new() -> Self {
        let f = Fixture::new();
        raw(&f.git, &["checkout", "-q", "--detach"]);
        raw(&f.git, &["branch", "integration", f.base.as_str()]);
        let bare = f.root.join("origin.git");
        raw(&f.git, &["init", "-q", "--bare", "--initial-branch=main", bare.to_str().unwrap()]);
        raw(&f.git, &["remote", "add", "origin", bare.to_str().unwrap()]);
        raw(&f.git, &["push", "-q", "origin", "refs/heads/main:refs/heads/main"]);
        Self { f, bare }
    }
    pub fn command(&self, cwd: &Path, words: &[&str]) -> Command {
        let mut command = self.f.command(cwd, &["--json", "--actor", "rit7"]);
        command.args(words); command
    }
    pub fn run(&self, cwd: &Path, words: &[&str], exit: i32, outcome: &str) -> Report {
        check(self.command(cwd, words).output().unwrap(), words[0], exit, outcome)
    }
    pub fn app(&self, words: &[&str], exit: i32, outcome: &str) -> Report {
        self.run(&self.f.git.repo, words, exit, outcome)
    }
    pub fn git_at(&self, path: &Path) -> Git { Git { repo: path.into(), ..self.f.git.clone() } }
    pub fn remote_refs(&self) -> Vec<u8> { raw(&self.git_at(&self.bare), &["for-each-ref", "--format=%(refname) %(objectname)"]) }
    pub fn refs(&self) -> LandRefs { LandRefs { origin_branch: "main".into(), integration_branch: "integration".into(), remote: "origin".into() } }
    pub fn locked<T>(&self, act: impl FnOnce() -> T) -> T {
        oplog::with_lock(&self.f.git, &self.f.common, &LockName::Land, "rit7-setup", &KillPidProbe, || {
            oplog::recover(&self.f.git, &self.f.common).unwrap(); Ok::<T, std::io::Error>(act())
        }).unwrap()
    }
    pub fn squash(&self, zone_tip: &Oid) -> Squash {
        raw(&self.f.git, &["update-ref", ZONE, zone_tip.as_str()]);
        self.locked(|| {
            let SquashAnswer::Made(squash) = land::squash(&self.f.git, &self.refs(), "core", "rit7-setup").unwrap() else { panic!("expected squash") };
            squash
        })
    }
    /// An actual Land fast-forward or merge published to the disposable remote, not a synthetic culprit result.
    pub fn failing_landing(&self, merge: bool) -> (Squash, Oid) {
        let zone = self.f.commit(&self.f.base, &[("src/file", b"broken\n")], "break the probe\n");
        let squash = self.squash(&zone);
        if merge {
            let origin = self.f.commit(&self.f.base, &[("src/peer", b"still passes\n")], "independent origin work\n");
            raw(&self.f.git, &["update-ref", "refs/heads/main", origin.as_str()]);
            raw(&self.f.git, &["update-ref", "refs/heads/integration", origin.as_str()]);
            raw(&self.f.git, &["push", "-q", "origin", "refs/heads/main:refs/heads/main"]);
        }
        let landed = self.locked(|| {
            let LandAnswer::Landed { integration } = land::land(&self.f.git, &self.refs(), &squash, "rit7-setup").unwrap() else { panic!("expected landing") };
            assert_eq!(land::publish(&self.f.git, &self.refs(), "rit7-setup").unwrap(), land::PublishAnswer::Published { origin: integration.clone() });
            integration
        });
        assert_eq!(self.f.git.commit_info(&landed).unwrap().parents.len(), if merge { 2 } else { 1 });
        (squash, landed)
    }
    pub fn bisect_command(&self, last: &Oid, failed: &Oid, timeout: &str, program: &Path, args: &[&str]) -> Command {
        let mut c = self.command(&self.f.git.repo, &["bisect", last.as_str(), failed.as_str(), "--timeout", timeout, "--", program.to_str().unwrap()]);
        c.args(args); c
    }
    /// Stable observable state; Git's transient probe worktree list must be identical after success or refusal.
    pub fn observe(&self) -> Vec<Vec<u8>> {
        let (refs, index, log, bytes) = self.f.observe();
        vec![refs, index, log.unwrap_or_default(), bytes, self.remote_refs(), raw(&self.f.git, &["worktree", "list", "--porcelain"])]
    }
    pub fn assert_no_probes_left(&self) {
        for entry in fs::read_dir(&self.f.root).unwrap() {
            let name = entry.unwrap().file_name(); assert!(!name.to_string_lossy().starts_with("mortar-bisect-"), "left a probe folder: {name:?}");
        }
        let common_entries: Vec<_> = fs::read_dir(&self.f.common).unwrap().map(|e| e.unwrap().file_name()).collect();
        assert!(!common_entries.iter().any(|n| n.to_string_lossy().starts_with("BISECT")), "left Git bisect state");
        assert!(!self.f.common.join("mortar/locks/land.lock").exists());
    }
    pub fn clone_store(&self) -> (PathBuf, Git) {
        let path = self.f.root.join("fresh-clone");
        raw(&self.f.git, &["clone", "-q", "--no-local", self.bare.to_str().unwrap(), path.to_str().unwrap()]);
        let git = self.git_at(&path); cli::write_config(&git.common_dir().unwrap(), &self.f.config).unwrap();
        (path, git)
    }
}
