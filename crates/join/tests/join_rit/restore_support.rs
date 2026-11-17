//! WO-join-rit-9: real join/resolve -> Gitio objects/ref I/O -> Oplog undo/valued log -> restore fixtures.
//! Reuses join_rit's disposable/config-isolated repository; no private Join state or codec is imported.

use super::support::Fixture;
use gitio::{Git, Oid, RunOpts};
use join::{JoinAnswer, JoinRunError, PausedJoin, RestoredJoin};
use oplog::{with_lock, ApplyOutcome, HeldLockError, KillPidProbe, LockName, OpCaller, OpPhase, ValuedRecord};
use std::{fs, path::PathBuf};

pub(super) const REQUEST: &str = "refs/heads/requests/rit";
pub(super) const ZONE: &str = "restore-rit";
pub(super) const ACTOR: &str = "restore-rit-caller";
const FILES: [&str; 5] = ["vendor/lib/file", "lib/x", "src/a.rs", "my \"odd\" file.txt", "ok.txt"];

pub(super) struct Scenario {
    pub f: Fixture,
    pub base: Oid,
    pub tip: Oid,
    pub zone: Oid,
    pub caller_tree: Oid,
    pub folder_tree: Oid,
}

impl Scenario {
    pub fn new(conflict: bool) -> Self {
        let f = Fixture::new();
        let base = f.git.read_ref("refs/heads/main").unwrap().unwrap();
        let tip_tree = tree(&f, &[("ok.txt", b"request\n")]);
        let tip = f.git.commit_tree(&tip_tree, std::slice::from_ref(&base), "request\n", None, None).unwrap();
        let mut zone_changes: Vec<(&str, &[u8])> = vec![("src/a.rs", b"zone outside\n")];
        if conflict { zone_changes.push(("ok.txt", b"zone\n")); }
        let zone_tree = tree(&f, &zone_changes);
        let zone = f.git.commit_tree(&zone_tree, std::slice::from_ref(&base), "zone\n", None, None).unwrap();
        for (name, value) in [(REQUEST.to_string(), &tip), (format!("refs/heads/zone/{ZONE}"), &zone)] {
            f.git.run(&["update-ref", &name, value.as_str()], &RunOpts::default()).unwrap();
        }
        let caller_tree = tree(&f, &[("ok.txt", b"resolved\n"), ("src/a.rs", b"caller outside\n")]);
        // Literal old request contents with just the resolved path replaced, not the advanced zone's tree.
        let folder_tree = tree(&f, &[("ok.txt", b"resolved\n")]);
        Self { f, base, tip, zone, caller_tree, folder_tree }
    }

    pub fn paused() -> Self {
        let r = Self::new(true);
        assert!(matches!(r.join().unwrap(), JoinAnswer::Conflicted { commit, files, .. }
            if commit == r.tip && files == vec![PathBuf::from("ok.txt")]));
        assert_eq!(r.resolve().unwrap(), JoinAnswer::FilesWritten {
            commit: r.tip.clone(), files: vec![PathBuf::from("ok.txt")], skipped: vec![], gitlinks: vec![],
        });
        fs::write(r.f.repo.path.join("ok.txt"), b"resolved\n").unwrap();
        fs::write(r.f.repo.path.join("src/a.rs"), b"caller outside\n").unwrap();
        r
    }

    pub fn common(&self) -> PathBuf { self.f.repo.path.join(".git") }
    pub fn state(&self) -> PathBuf { self.common().join("mortar-join.json") }
    pub fn state_bytes(&self) -> Vec<u8> { fs::read(self.state()).unwrap() }

    pub fn call<T>(&self, work: impl FnOnce() -> Result<T, JoinRunError>) -> Result<T, JoinRunError> {
        caller(&self.f.git, &self.f.repo.path, work)
    }

    pub fn join(&self) -> Result<JoinAnswer, JoinRunError> {
        self.call(|| join::join(&self.f.git, &self.f.repo.path, ZONE, REQUEST, ACTOR))
    }

    pub fn resolve(&self) -> Result<JoinAnswer, JoinRunError> {
        self.call(|| join::resolve(&self.f.git, &self.f.repo.path, &self.caller_tree, ACTOR))
    }

    pub fn restore(&self, op: &ValuedRecord) -> Result<RestoredJoin, JoinRunError> {
        self.call(|| join::restore_join_state(&self.f.git, &self.f.repo.path, op))
    }

    pub fn undo(&self, op: &ValuedRecord) {
        let caller = OpCaller { key: format!("undo:{}", op.record.op_id), actor: ACTOR.into(), reason: "undo resolve".into() };
        let outcome = self.call(|| oplog::undo(&self.f.git, &self.common(), &[REQUEST.into()], &caller, &[])
            .map_err(JoinRunError::Oplog)).unwrap();
        let ApplyOutcome::Done(record) = outcome else { panic!("expected completed undo: {outcome:?}") };
        assert_eq!(record.kind, "undo");
        assert_eq!(record.moves, vec![oplog::RefMove {
            refname: REQUEST.into(), old: Some(op.record.moves[0].new.clone()), new: self.tip.clone(),
        }]);
        assert_eq!(self.f.git.read_ref(REQUEST).unwrap(), Some(self.tip.clone()));
    }

    pub fn done(&self) -> ValuedRecord {
        let log = oplog::read_valued_log(&self.common()).unwrap();
        let mut records = log.records.into_iter().filter(|v| v.record.kind == "join-rebase" && v.record.phase == OpPhase::Done);
        let op = records.next().expect("real join-rebase done record");
        assert!(records.next().is_none(), "setup must contain exactly one completed rebase");
        op
    }

    pub fn assert_values(&self, op: &ValuedRecord, before: &[u8]) {
        let before = std::str::from_utf8(before).unwrap().strip_suffix('\n').expect("state newline");
        assert_eq!(op.values, vec![
            ("folder_tree".into(), self.folder_tree.to_string()), ("join_state".into(), before.to_string()),
        ]);
        assert_eq!(op.record.moves.len(), 1);
        assert_eq!(op.record.moves[0].refname, REQUEST);
        assert_eq!(op.record.moves[0].old, Some(self.tip.clone()));
    }

    pub fn assert_rebased(&self, answer: JoinAnswer) -> Oid {
        let JoinAnswer::Rebased { request_ref, from, to, folder_tree } = answer else {
            panic!("expected completed resolve: {answer:?}")
        };
        assert_eq!(request_ref, REQUEST); assert_eq!(from, self.tip);
        assert_eq!(folder_tree, Some(self.folder_tree.clone()));
        let info = self.f.git.commit_info(&to).unwrap();
        assert_eq!(info.parents, vec![self.zone.clone()]);
        assert_eq!(info.tree, tree(&self.f, &[("ok.txt", b"resolved\n"), ("src/a.rs", b"zone outside\n")]));
        assert_eq!(self.f.git.read_ref(REQUEST).unwrap(), Some(to.clone()));
        assert_eq!(self.f.git.read_ref(&format!("refs/heads/zone/{ZONE}")).unwrap(), Some(self.zone.clone()));
        assert!(!self.state().exists());
        to
    }

    pub fn restored(&self, repeated: bool) -> RestoredJoin {
        let paused = PausedJoin { commit: self.tip.clone(), files: vec![PathBuf::from("ok.txt")] };
        if repeated { RestoredJoin::AlreadyRestored { state: self.state(), paused } }
        else { RestoredJoin::Restored { state: self.state(), paused } }
    }

    /// State is checked separately; everything else must survive a restore/refusal byte for byte.
    pub fn observe(&self) -> Vec<Vec<u8>> {
        let mut bytes = vec![
            self.f.git.run(&["for-each-ref", "--format=%(refname) %(objectname)"], &RunOpts::default()).unwrap().stdout,
            self.f.git.run(&["rev-parse", "HEAD"], &RunOpts::default()).unwrap().stdout,
            fs::read(self.common().join("index")).unwrap(),
            fs::read(oplog::oplog_path(&self.common())).unwrap_or_default(),
        ];
        bytes.extend(FILES.iter().map(|name| fs::read(self.f.repo.path.join(name)).unwrap()));
        bytes
    }
}

pub(super) fn caller<T>(git: &Git, folder: &std::path::Path, work: impl FnOnce() -> Result<T, JoinRunError>)
    -> Result<T, JoinRunError>
{
    let common = git.common_dir().unwrap();
    assert_eq!(git.git_dir(folder).unwrap(), common);
    match with_lock(git, &common, &LockName::Join(ZONE.into()), ACTOR, &KillPidProbe, || {
        oplog::recover(git, &common).map_err(JoinRunError::Oplog)?; work()
    }) {
        Ok(value) => Ok(value), Err(HeldLockError::Work(error)) => Err(error), Err(other) => panic!("caller lock: {other:?}"),
    }
}

/// Raw Git builds an independent literal-contents oracle, not Join's folder-tree composition routine.
pub(super) fn tree(f: &Fixture, changes: &[(&str, &[u8])]) -> Oid {
    let opts = RunOpts { env: vec![("GIT_INDEX_FILE".into(), f.repo.path.join(".git/restore-index").into_os_string())],
        ..RunOpts::default() };
    f.git.run(&["read-tree", f.tree.as_str()], &opts).unwrap();
    for (path, bytes) in changes {
        let blob = f.git.run(&["hash-object", "-w", "--stdin"], &RunOpts { stdin: Some(bytes.to_vec()), ..opts.clone() }).unwrap();
        let blob = std::str::from_utf8(&blob.stdout).unwrap().trim_end();
        f.git.run(&["update-index", "--add", "--cacheinfo", "100644", blob, path], &opts).unwrap();
    }
    let out = f.git.run(&["write-tree"], &opts).unwrap();
    Oid::parse(std::str::from_utf8(&out.stdout).unwrap().trim_end()).unwrap()
}
