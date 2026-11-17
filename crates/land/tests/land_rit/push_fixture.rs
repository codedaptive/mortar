//! WO-land-rit-3 fixtures: public Land producers joined to real Gitio/Oplog and a disposable bare origin.
//! Existing support owns/removes the whole repository, including this fixture's nested remote and shims.
use crate::{owed_ops::{locked, raw, refs}, support::Fixture};
use gitio::{Git, Oid};
use land::{LandAnswer, PublishAnswer, Squash, SquashAnswer};
use oplog::{KillPidProbe, LockName, MarkedRecord};
use std::{fmt::Debug, path::PathBuf};

pub const ACTOR: &str = "land-rit-3";
pub const MAIN: &str = "refs/heads/main";
pub const ZONE: &str = "refs/heads/zone/core";
pub const SQUASH: &str = "refs/mortar/squash/core";
pub struct Prepared { pub f: Fixture, pub base: Oid, pub new: Oid, pub squash: Squash, pub remote: Git }
impl Prepared {
    pub fn new() -> Self {
        let f = Fixture::new(); let base = f.git.read_ref(MAIN).unwrap().unwrap();
        let remote_path = f.git.repo.join("rit3-remote.git");
        raw(&f, &["init", "--bare", "--quiet", remote_path.to_str().unwrap()]);
        raw(&f, &["remote", "add", "origin", remote_path.to_str().unwrap()]);
        raw(&f, &["push", "--quiet", "origin", &format!("{base}:{MAIN}")]);
        let SquashAnswer::Made(squash) = locked(&f, || land::squash(&f.git, &refs(), "core", ACTOR)) else {
            panic!("fresh fixture must squash");
        };
        let new = squash.commit.clone();
        assert_eq!(locked(&f, || land::land(&f.git, &refs(), &squash, ACTOR)), LandAnswer::Landed { integration: new.clone() });
        let remote = Git { repo: remote_path, ..f.git.clone() };
        Self { f, base, new, squash, remote }
    }
    pub fn publish(&self, git: &Git) -> PublishAnswer {
        with_lock(self, || { oplog::recover(git, &self.f.common).unwrap(); land::publish(git, &refs(), ACTOR) })
    }
    pub fn remote_tip(&self) -> Option<Oid> { self.remote.read_ref(MAIN).unwrap() }
    pub fn state(&self) -> PathBuf { self.f.common.join("mortar/landing.json") }
    pub fn pushes(&self) -> Vec<MarkedRecord> {
        oplog::read_marked_log(&self.f.common).unwrap().records.into_iter()
            .filter(|m| m.record.kind == "publish-push").collect()
    }
    /// An actual external push. Keeping the parent's tree avoids an unrelated merge-conflict question.
    pub fn foreign_push_on(&self, parent: &Oid) -> Oid {
        let tree = self.f.git.commit_info(parent).unwrap().tree;
        let next = self.f.git.commit_tree(&tree, std::slice::from_ref(parent), "external actor\n", None, None).unwrap();
        raw(&self.f, &["push", "--quiet", "origin", &format!("{next}:{MAIN}")]); next
    }
    pub fn traced_git(&self) -> Git {
        let mut git = self.f.git.clone(); git.program = self.f.git.repo.join("rit3-push-git");
        git.env.push(("RIT_PUSH_TRACE".into(), self.trace().into_os_string()));
        git.env.push(("RIT_REAL_GIT".into(), self.f.git.program.clone().into_os_string())); git
    }
    pub fn trace(&self) -> PathBuf { self.f.git.repo.join("rit3-push-trace") }
    pub fn trace_bytes(&self) -> Vec<u8> { std::fs::read(self.trace()).unwrap_or_else(|e| {
        assert_eq!(e.kind(), std::io::ErrorKind::NotFound); vec![]
    }) }
    pub fn assert_finished(&self, remote: &Oid) {
        assert_eq!(self.f.git.read_ref(MAIN).unwrap(), Some(self.new.clone()));
        assert_eq!(self.f.git.read_ref("refs/heads/integration").unwrap(), Some(self.new.clone()));
        assert_eq!(self.remote_tip(), Some(remote.clone()));
        assert!(!self.state().exists()); assert_eq!(self.f.git.read_ref(SQUASH).unwrap(), None);
    }
    pub fn assert_inert_repeat(&self, git: &Git) {
        let log = oplog::read_marked_log(&self.f.common).unwrap().records;
        let trace = self.trace_bytes(); let remote = self.remote_tip();
        let local_refs = || self.f.git.run(&["for-each-ref"], &gitio::RunOpts::default()).unwrap().stdout;
        let before_refs = local_refs();
        assert_eq!(self.publish(git), PublishAnswer::NothingToPublish);
        assert_eq!(local_refs(), before_refs); assert!(!self.state().exists());
        assert_eq!(oplog::read_marked_log(&self.f.common).unwrap().records, log);
        assert_eq!(self.trace_bytes(), trace); assert_eq!(self.remote_tip(), remote);
    }
}
pub fn with_lock<T, E: Debug>(p: &Prepared, work: impl FnOnce() -> Result<T, E>) -> T {
    oplog::with_lock(&p.f.git, &p.f.common, &LockName::Land, "rit3-lock", &KillPidProbe, work).unwrap()
}
