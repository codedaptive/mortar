use gitio::{Git, Oid, RunOpts};
use join::{BaseJoinAnswer, JoinRunError, Partner, PartnerMerge, RequestBase};
use oplog::{with_lock, KillPidProbe, LockName};
use testkit::TestRepo;

pub const ZONE: &str = "zit-zone";
pub const ZONE_REF: &str = "refs/heads/zone/zit-zone";
pub const P1_REF: &str = "refs/heads/zone/p1";
pub const P2_REF: &str = "refs/heads/zone/p2";
pub const REQUEST_REF: &str = "refs/heads/request/zit";
pub const ACTOR: &str = "partner-base-caller";

pub struct Fixture { pub repo: TestRepo, pub git: Git }

impl Fixture {
    pub fn new() -> Self {
        let repo = TestRepo::new().unwrap();
        let mut git = Git::new(&repo.path);
        git.env.extend([
            ("GIT_CONFIG_NOSYSTEM".into(), "1".into()), ("GIT_CONFIG_GLOBAL".into(), "/dev/null".into()),
            ("GIT_AUTHOR_NAME".into(), "Author".into()), ("GIT_AUTHOR_EMAIL".into(), "author@example.invalid".into()),
            ("GIT_AUTHOR_DATE".into(), "1700000000 +0530".into()),
            ("GIT_COMMITTER_NAME".into(), "Committer".into()), ("GIT_COMMITTER_EMAIL".into(), "committer@example.invalid".into()),
            ("GIT_COMMITTER_DATE".into(), "1700000999 +0000".into()),
        ]);
        Self { repo, git }
    }
    pub fn run(&self, args: &[&str], stdin: Option<&[u8]>, index: bool) -> String {
        let mut opts = RunOpts { stdin: stdin.map(<[u8]>::to_vec), ..RunOpts::default() };
        if index { opts.env = vec![("GIT_INDEX_FILE".into(), self.repo.path.join(".git/oracle-index").into_os_string())]; }
        String::from_utf8(self.git.run(args, &opts).unwrap().stdout).unwrap()
    }
    pub fn oid(&self, args: &[&str], stdin: Option<&[u8]>, index: bool) -> Oid {
        Oid::parse(self.run(args, stdin, index).trim_end()).unwrap()
    }
    pub fn tree(&self, files: &[(&str, &[u8])]) -> Oid {
        self.run(&["read-tree", "--empty"], None, true);
        for (path, bytes) in files {
            let blob = self.oid(&["hash-object", "-w", "--stdin"], Some(bytes), false);
            self.run(&["update-index", "--add", "--cacheinfo", "100644", blob.as_str(), path], None, true);
        }
        self.oid(&["write-tree"], None, true)
    }
    pub fn commit(&self, files: &[(&str, &[u8])], parent: Option<&Oid>, message: &str) -> Oid {
        let tree = self.tree(files);
        let mut args = vec!["commit-tree", tree.as_str()];
        if let Some(parent) = parent { args.extend(["-p", parent.as_str()]); }
        self.oid(&args, Some(message.as_bytes()), false)
    }
    pub fn update(&self, name: &str, oid: &Oid) { self.run(&["update-ref", name, oid.as_str()], None, false); }
    pub fn checkout(&self, zone: &Oid, request: &Oid) {
        self.update(ZONE_REF, zone); self.update(REQUEST_REF, request);
        self.run(&["checkout", "--force", "request/zit"], None, false);
    }
    pub fn refs(&self) -> String { self.run(&["for-each-ref", "--format=%(refname) %(objectname)"], None, false) }
    pub fn caller<T>(&self, work: impl FnOnce() -> Result<T, JoinRunError>) -> T {
        let common = self.repo.path.join(".git");
        match with_lock(&self.git, &common, &LockName::Join(ZONE.into()), ACTOR, &KillPidProbe, || {
            oplog::recover(&self.git, &common).map_err(JoinRunError::Oplog)?; work()
        }) {
            Ok(value) => value, Err(other) => panic!("caller: {other:?}"),
        }
    }
    pub fn join(&self, base: &RequestBase) -> BaseJoinAnswer {
        self.caller(|| join::join_on_base(&self.git, &self.repo.path, ZONE, REQUEST_REF, base, ACTOR))
    }
    pub fn merge(&self, zone: &Oid, partners: &[Partner]) -> RequestBase {
        match join::make_partner_merge(&self.git, zone, partners).unwrap() {
            PartnerMerge::Merged(base) => base, other => panic!("expected merge: {other:?}"),
        }
    }
    pub fn state_exists(&self) -> bool { self.repo.path.join(".git/mortar-join.json").exists() }
}

pub fn partner(zone: &str, tip: &Oid) -> Partner { Partner { zone: zone.into(), tip: tip.clone() } }

pub struct Scenario { pub f: Fixture, pub zone: Oid, pub p1: Oid, pub base: RequestBase, pub request: Oid }
impl Scenario {
    pub fn new(conflict_own: bool) -> Self {
        let f = Fixture::new();
        let root = f.commit(&[("shared", b"base\n")], None, "root\n");
        let zone = f.commit(&[("shared", b"base\n"), ("zone", b"zone\n")], Some(&root), "zone\n");
        let p1 = f.commit(&[("shared", b"base\n"), ("partner", b"p1\n")], Some(&root), "partner\n");
        f.update(P1_REF, &p1);
        let base = f.merge(&zone, &[partner("p1", &p1)]);
        let shared: &[u8] = if conflict_own { b"request\n" } else { b"base\n" };
        let request = f.commit(&[("shared", shared), ("zone", b"own\n"), ("partner", b"p1\n")], Some(&base.base), "own\n");
        f.checkout(&zone, &request);
        Self { f, zone, p1, base, request }
    }
}
