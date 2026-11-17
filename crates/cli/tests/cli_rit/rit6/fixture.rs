//! Real-object setup shared by the WO-cli-rit-6 interface tests; never used against an existing repository.
use crate::{json::{check, Report}, support::{raw, write, Fixture, ZONE}};
use gitio::{Git, Oid};
use land::{LandAnswer, LandRefs, Squash, SquashAnswer};
use oplog::{KillPidProbe, LockName};
use std::{fs, path::PathBuf, process::{Command, Output}};

pub struct Landing {
    pub f: Fixture,
    pub refs: LandRefs,
    pub squash: Squash,
    pub folder: PathBuf,
    pub bare: PathBuf,
}
impl Landing {
    pub fn new(with_folder: bool) -> Self {
        let f = Fixture::new();
        raw(&f.git, &["checkout", "-q", "--detach"]);
        raw(&f.git, &["branch", "integration", f.base.as_str()]);
        let zone = f.commit(&f.base, &[("src/file", b"zone work\n")], "zone work\n");
        raw(&f.git, &["update-ref", ZONE, zone.as_str()]);
        let refs = LandRefs { origin_branch: "main".into(), integration_branch: "integration".into(), remote: "origin".into() };
        let squash = locked(&f, || {
            let SquashAnswer::Made(squash) = land::squash(&f.git, &refs, "core", "rit6-setup").unwrap() else { panic!("expected squash") };
            squash
        });
        let folder = f.root.join("integration");
        if with_folder { raw(&f.git, &["worktree", "add", "-q", folder.to_str().unwrap(), "integration"]); }
        let bare = f.root.join("origin.git");
        raw(&f.git, &["init", "-q", "--bare", "--initial-branch=main", bare.to_str().unwrap()]);
        raw(&f.git, &["remote", "add", "origin", bare.to_str().unwrap()]);
        raw(&f.git, &["push", "-q", "origin", "refs/heads/main:refs/heads/main"]);
        Self { f, refs, squash, folder, bare }
    }
    pub fn command(&self, verb: &str, word: Option<&str>) -> Command {
        let mut command = self.f.command(&self.f.git.repo, &["--json", "--actor", "rit6", verb]);
        if let Some(word) = word { command.arg(word); }
        command
    }
    pub fn output(&self, verb: &str, word: Option<&str>) -> Output { self.command(verb, word).output().unwrap() }
    pub fn run(&self, verb: &str, word: Option<&str>, exit: i32, outcome: &str) -> Report {
        check(self.output(verb, word), verb, exit, outcome)
    }
    pub fn land(&self) -> Report { self.run("land", Some(self.squash.commit.as_str()), 0, "landed") }
    pub fn tip(&self) -> Oid { self.f.git.read_ref("refs/heads/integration").unwrap().unwrap() }
    pub fn folder_git(&self) -> Git { Git { repo: self.folder.clone(), ..self.f.git.clone() } }
    pub fn index(&self) -> Vec<u8> { raw(&self.folder_git(), &["ls-files", "--stage", "-z"]) }
    pub fn bytes(&self) -> Vec<u8> { fs::read(self.folder.join("src/file")).unwrap() }
    pub fn sync(&self, old: &Oid, new: &Oid) {
        raw(&self.folder_git(), &["read-tree", "-m", "-u", old.as_str(), new.as_str()]);
    }
    pub fn locked<T>(&self, act: impl FnOnce() -> T) -> T { locked(&self.f, act) }
    pub fn kit_land(&self) {
        assert_eq!(self.locked(|| land::land(&self.f.git, &self.refs, &self.squash, "rit6-setup").unwrap()),
            LandAnswer::Landed { integration: self.squash.commit.clone() });
    }
    pub fn kit_back_out(&self) {
        assert!(matches!(self.locked(|| land::back_out(&self.f.git, &self.refs, "rit6-setup").unwrap()), land::BackOutAnswer::BackedOut { .. }));
    }
    pub fn remote_move(&self) -> Oid {
        let next = self.f.commit(&self.f.base, &[("src/file", b"remote work\n")], "remote advanced\n");
        raw(&self.f.git, &["push", "-q", "origin", &format!("{next}:refs/heads/main")]);
        next
    }
    pub fn peer_squash(&self) -> Squash {
        let commit = self.f.commit(&self.f.base, &[("src/peer", b"peer work\n")], "peer work\n");
        raw(&self.f.git, &["update-ref", "refs/heads/zone/peer", commit.as_str()]);
        self.locked(|| {
            let SquashAnswer::Made(squash) = land::squash(&self.f.git, &self.refs, "peer", "rit6-peer").unwrap() else { panic!("expected peer squash") };
            squash
        })
    }
    /// An independent state oracle: all refs, op-log bytes, landing state, remote refs, index entries and file bytes.
    pub fn observe(&self) -> Vec<Vec<u8>> {
        let mut state = vec![raw(&self.f.git, &["for-each-ref", "--format=%(refname) %(objectname)"]),
            self.f.log().unwrap_or_default(), fs::read(self.f.common.join("mortar/landing.json")).unwrap_or_default(),
            raw(&Git { repo: self.bare.clone(), ..self.f.git.clone() }, &["for-each-ref", "--format=%(refname) %(objectname)"])];
        if self.folder.exists() { state.extend([self.index(), self.bytes()]); }
        state
    }
    pub fn assert_finished(&self, report: &Report, old: &Oid, tip: &Oid) {
        assert_eq!(report.strings("lines")[0], format!("first finished a stopped move: the integration folder {} moved from {old} to {tip}", self.folder.display()));
    }
    pub fn assert_clean_at(&self, tip: &Oid, bytes: &[u8]) {
        assert_eq!(self.tip(), *tip);
        assert_eq!(self.f.git.index_tree(&self.folder).unwrap(), self.f.git.commit_info(tip).unwrap().tree);
        assert_eq!(self.bytes(), bytes);
        assert_eq!(raw(&self.folder_git(), &["status", "--porcelain"]), b"");
    }
    pub fn obstruct(&self) { write(&self.folder, "src/file", b"owner's unsaved work\n"); }
    pub fn clear_to_index(&self) {
        let bytes = raw(&self.folder_git(), &["show", ":src/file"]);
        write(&self.folder, "src/file", &bytes);
    }
}
fn locked<T>(f: &Fixture, act: impl FnOnce() -> T) -> T {
    oplog::with_lock(&f.git, &f.common, &LockName::Land, "rit6-setup", &KillPidProbe, || {
        oplog::recover(&f.git, &f.common).unwrap();
        Ok::<T, std::io::Error>(act())
    }).unwrap()
}
