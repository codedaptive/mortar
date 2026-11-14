//! WO-join-3e, Orc #1311/#1461, S-84/S-87/S-89, rule 11 (ZIT): public join::join, join::resolve and
//! join::paused_join with real git in one throwaway repository; the op log read with oplog::read_valued_log.
use gitio::{Git, Oid, RunOpts};
use join::{JoinAnswer, JoinRunError, PausedJoin};
use oplog::{with_lock, KillPidProbe, LockName, OpPhase};
use std::{fs, path::PathBuf};
use testkit::TestRepo;

const ZONE: &str = "zit-zone";
const ZONE_REF: &str = "refs/heads/zone/zit-zone";
const REQUEST_REF: &str = "refs/heads/request/zit";
const ACTOR: &str = "real-join-caller";
const P: &str = "shared.txt";

struct Fixture { repo: TestRepo, git: Git }

impl Fixture {
    fn new() -> Self {
        let repo = TestRepo::new().unwrap();
        let mut git = Git::new(&repo.path);
        git.env.extend([
            ("GIT_CONFIG_NOSYSTEM".into(), "1".into()), ("GIT_CONFIG_GLOBAL".into(), "/dev/null".into()),
            ("GIT_AUTHOR_NAME".into(), "Author".into()), ("GIT_AUTHOR_EMAIL".into(), "author@example.invalid".into()),
            ("GIT_AUTHOR_DATE".into(), "1700000000 +0000".into()),
            ("GIT_COMMITTER_NAME".into(), "Committer".into()), ("GIT_COMMITTER_EMAIL".into(), "committer@example.invalid".into()),
            ("GIT_COMMITTER_DATE".into(), "1700000999 +0000".into()),
        ]);
        Self { repo, git }
    }
    fn run(&self, args: &[&str], stdin: Option<&[u8]>, index: bool) -> String {
        let mut opts = RunOpts { stdin: stdin.map(<[u8]>::to_vec), ..RunOpts::default() };
        if index { opts.env = vec![("GIT_INDEX_FILE".into(), self.repo.path.join(".git/oracle-index").into_os_string())]; }
        String::from_utf8(self.git.run(args, &opts).unwrap().stdout).unwrap()
    }
    fn oid(&self, args: &[&str], stdin: Option<&[u8]>, index: bool) -> Oid { Oid::parse(self.run(args, stdin, index).trim_end()).unwrap() }
    /// The tree holding exactly `files`, built with git plumbing: the oracle every expected tree comes from.
    fn tree(&self, files: &[(&str, &[u8])]) -> Oid {
        self.run(&["read-tree", "--empty"], None, true);
        for (path, bytes) in files {
            let blob = self.oid(&["hash-object", "-w", "--stdin"], Some(bytes), false);
            self.run(&["update-index", "--add", "--cacheinfo", "100644", blob.as_str(), path], None, true);
        }
        self.oid(&["write-tree"], None, true)
    }
    fn commit(&self, files: &[(&str, &[u8])], parent: Option<&Oid>, message: &str) -> Oid {
        let tree = self.tree(files);
        let mut args = vec!["commit-tree", tree.as_str()];
        if let Some(parent) = parent { args.extend(["-p", parent.as_str()]); }
        self.oid(&args, Some(message.as_bytes()), false)
    }
    /// Points the zone and request refs at `zone` and `request` and checks the folder out on the request branch.
    fn refs(&self, zone: &Oid, request: &Oid) {
        self.run(&["update-ref", ZONE_REF, zone.as_str()], None, false);
        self.run(&["update-ref", REQUEST_REF, request.as_str()], None, false);
        self.run(&["checkout", "--force", "request/zit"], None, false);
    }
    /// As a caller does: holds the join lock and runs recover before `work`.
    fn caller<T>(&self, work: impl FnOnce() -> Result<T, JoinRunError>) -> T {
        let common = self.repo.path.join(".git");
        match with_lock(&self.git, &common, &LockName::Join(ZONE.into()), ACTOR, &KillPidProbe, || {
            oplog::recover(&self.git, &common).map_err(JoinRunError::Oplog)?; work()
        }) {
            Ok(value) => value, Err(other) => panic!("caller: {other:?}"),
        }
    }
    fn join(&self) -> JoinAnswer { self.caller(|| join::join(&self.git, &self.repo.path, ZONE, REQUEST_REF, ACTOR)) }
    fn resolve(&self, tree: &Oid) -> JoinAnswer { self.caller(|| join::resolve(&self.git, &self.repo.path, tree, ACTOR)) }
    /// The join-rebase records in the op log, read as oplog's public reader reads them: (phase, values).
    fn rebase_records(&self) -> Vec<(OpPhase, Vec<(String, String)>)> {
        let log = oplog::read_valued_log(&self.repo.path.join(".git")).unwrap();
        log.records.into_iter().filter(|r| r.record.kind == "join-rebase").map(|r| (r.record.phase, r.values)).collect()
    }
}

/// A request commit and a zone commit that change P two ways from one base; join stops on P.
fn paused() -> (Fixture, Oid) {
    let f = Fixture::new();
    let base = f.commit(&[(P, b"base\n"), ("keep", b"keep\n")], None, "base\n");
    let request = f.commit(&[(P, b"request\n"), ("keep", b"keep\n")], Some(&base), "request change\n");
    let zone = f.commit(&[(P, b"zone\n"), ("keep", b"keep\n")], Some(&base), "zone change\n");
    f.refs(&zone, &request);
    (f, request)
}

#[test]
fn a_join_paused_on_a_conflict_is_answered_by_paused_join() {
    // Joins join -> replay's conflict -> pause's mortar-join.json -> paused_join reading it back (S-84).
    let (f, request) = paused();
    assert!(matches!(f.join(), JoinAnswer::Conflicted { .. }));
    let expected = PausedJoin { commit: request, files: vec![PathBuf::from(P)] };
    assert_eq!(join::paused_join(&f.repo.path.join(".git")).unwrap(), Some(expected));
}

#[test]
fn a_completed_resolve_records_folder_tree_on_its_final_op() {
    // Joins join's pause -> resolve's first run -> resolve's commit, folder tree and final op (S-87, S-89).
    let (f, _) = paused();
    assert!(matches!(f.join(), JoinAnswer::Conflicted { .. }));
    let caller_tree = f.tree(&[(P, b"resolved\n"), ("keep", b"caller edit outside the conflict\n")]);
    assert!(matches!(f.resolve(&caller_tree), JoinAnswer::FilesWritten { .. }));
    fs::write(f.repo.path.join(P), b"resolved\n").unwrap();
    let JoinAnswer::Rebased { folder_tree: Some(t), .. } = f.resolve(&caller_tree) else { panic!("expected Rebased with a folder_tree") };
    // S-87: the old request tip's tree with P taken from the caller's tree; the caller's edit to keep stays out.
    assert_eq!(t, f.tree(&[(P, b"resolved\n"), ("keep", b"keep\n")]));
    let values = vec![("folder_tree".to_string(), t.to_string())];
    assert_eq!(f.rebase_records(), vec![(OpPhase::Intent, values.clone()), (OpPhase::Done, values)]);
}

#[test]
fn a_plain_join_rebase_op_carries_no_values() {
    // Joins join -> clean replay -> rebase_op applied: the op names no values (S-89).
    let f = Fixture::new();
    let base = f.commit(&[(P, b"base\n")], None, "base\n");
    let request = f.commit(&[(P, b"request\n")], Some(&base), "request change\n");
    let zone = f.commit(&[(P, b"base\n"), ("zone-only", b"zone\n")], Some(&base), "zone change\n");
    f.refs(&zone, &request);
    assert!(matches!(f.join(), JoinAnswer::Rebased { folder_tree: None, .. }));
    assert_eq!(f.rebase_records(), vec![(OpPhase::Intent, vec![]), (OpPhase::Done, vec![])]);
}
