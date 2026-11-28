use gitio::{Git, Oid, RunOpts};
use land::{land_batch, BatchLandAnswer, LandRefs, Squash};
use oplog::{oplog_path, read_op_log, ChangeId, OpPhase};
use std::ffi::OsString;
use std::path::PathBuf;
use testkit::TestRepo;

pub const ID: &str = "klmnopqrstuvwxyzklmnopqrstuvwxyz";
pub const ACTOR: &str = "zit@host";
pub const DATE: &str = "2000-01-01T00:00:00+00:00";
pub const RECEIPT: &str = "validation/r.txt";

pub fn git_env(date: &str) -> Vec<(OsString, OsString)> {
    [
        ("GIT_CONFIG_NOSYSTEM", "1"), ("GIT_CONFIG_GLOBAL", "/dev/null"),
        ("GIT_AUTHOR_NAME", "Zit"), ("GIT_AUTHOR_EMAIL", "zit@example.invalid"),
        ("GIT_AUTHOR_DATE", date), ("GIT_COMMITTER_NAME", "Zit"),
        ("GIT_COMMITTER_EMAIL", "zit@example.invalid"), ("GIT_COMMITTER_DATE", date),
    ].map(|(k, v)| (k.into(), v.into())).to_vec()
}

pub fn refs() -> LandRefs {
    LandRefs { origin_branch: "main".into(), integration_branch: "integration".into(), remote: "origin".into() }
}

pub fn git_at(path: PathBuf) -> Git {
    let mut git = Git::new(path);
    git.env = git_env(DATE);
    git
}

pub fn run(git: &Git, args: &[&str]) -> String {
    String::from_utf8(git.run(args, &RunOpts::default()).unwrap().stdout).unwrap()
}

fn tree(git: &Git, files: &[(&str, &str)]) -> Oid {
    let mut listing = String::new();
    for (name, content) in files {
        let out = git.run(&["hash-object", "-w", "--stdin"], &RunOpts {
            stdin: Some(content.as_bytes().to_vec()), ..RunOpts::default()
        }).unwrap();
        listing.push_str(&format!("100644 blob {}\t{name}\n", String::from_utf8(out.stdout).unwrap().trim()));
    }
    let out = git.run(&["mktree"], &RunOpts { stdin: Some(listing.into_bytes()), ..RunOpts::default() }).unwrap();
    Oid::parse(String::from_utf8(out.stdout).unwrap().trim()).unwrap()
}

pub struct Fixture {
    pub repo: TestRepo,
    /// Holds the bare remote and the integration folder; kept only so they are removed with the fixture.
    _elsewhere: TestRepo,
    pub git: Git,
    pub remote: Git,
    pub common: PathBuf,
    pub folder: PathBuf,
    pub base: Oid,
}

impl Fixture {
    pub fn new() -> Self {
        let repo = TestRepo::new().unwrap();
        let git = git_at(repo.path.clone());
        let base = git.commit_tree(&tree(&git, &[("a.txt", "base\n")]), &[], "base\n", None, None).unwrap();
        let elsewhere = TestRepo::new().unwrap();
        let bare = elsewhere.path.join("remote.git");
        run(&git_at(elsewhere.path.clone()), &["init", "--bare", "--object-format=sha1", bare.to_str().unwrap()]);
        let remote = git_at(bare.clone());
        let common = repo.path.join(".git");
        let folder = elsewhere.path.join("integration");
        let fx = Self { repo, _elsewhere: elsewhere, git, remote, common, folder, base };
        for name in ["main", "integration"] {
            run(&fx.git, &["update-ref", &format!("refs/heads/{name}"), fx.base.as_str()]);
        }
        run(&fx.git, &["remote", "add", "origin", bare.to_str().unwrap()]);
        run(&fx.git, &["push", "origin", "refs/heads/main:refs/heads/main"]);
        fx
    }

    pub fn add_folder(&self) -> Git {
        run(&self.git, &["worktree", "add", self.folder.to_str().unwrap(), "integration"]);
        git_at(self.folder.clone())
    }

    pub fn landed() -> (Self, Squash, Git) {
        let fx = Self::new();
        let message = format!("land: squash of r1\n\nMortar-Change-Id: {ID}\nMortar-Zone: land\nMortar-Request: r1\n");
        let squash = Squash {
            commit: fx.git.commit_tree(&tree(&fx.git, &[("a.txt", "base\n"), ("s.txt", "squash\n")]),
                std::slice::from_ref(&fx.base), &message, None, None).unwrap(),
            change_id: ChangeId::parse(ID).unwrap(), source: fx.base.clone(), requests: vec!["r1".into()],
        };
        run(&fx.git, &["update-ref", "refs/mortar/squash/land", squash.commit.as_str()]);
        assert_eq!(land_batch(&fx.git, &refs(), &squash, ACTOR).unwrap(), BatchLandAnswer::Landed { integration: squash.commit.clone() });
        let folder_git = fx.add_folder();
        (fx, squash, folder_git)
    }

    pub fn branch(&self, name: &str) -> Oid {
        self.git.read_ref(&format!("refs/heads/{name}")).unwrap().unwrap()
    }
    pub fn state(&self) -> Option<String> {
        std::fs::read_to_string(self.common.join("mortar/landing.json")).ok()
    }
    pub fn log(&self) -> Option<Vec<u8>> {
        std::fs::read(oplog_path(&self.common)).ok()
    }
    pub fn phases(&self) -> Vec<OpPhase> {
        read_op_log(&self.common).unwrap().records.into_iter()
            .filter(|r| r.kind == "land-commit-files").map(|r| r.phase).collect()
    }
    pub fn receipt(&self) {
        std::fs::create_dir_all(self.folder.join("validation")).unwrap();
        std::fs::write(self.folder.join(RECEIPT), "receipt\n").unwrap();
    }
    pub fn assert_commit(&self, commit: &Oid, parent: &Oid) {
        assert_eq!(self.git.commit_info(commit).unwrap().parents, vec![parent.clone()]);
        assert_eq!(run(&self.git, &["ls-tree", "-r", "--name-only", commit.as_str()]), "a.txt\ns.txt\nvalidation/r.txt\n");
        for (path, content) in [("a.txt", "base\n"), ("s.txt", "squash\n"), (RECEIPT, "receipt\n")] {
            assert_eq!(run(&self.git, &["show", &format!("{commit}:{path}")]), content);
        }
    }
    pub fn state_text(&self, squash: &Oid, integration: &Oid, pending: Option<&Oid>) -> String {
        let tail = pending.map(|p| format!(",\"pending\":\"{p}\"")).unwrap_or_default();
        format!("{{\"change_id\":\"{ID}\",\"zone\":\"land\",\"squash\":\"{squash}\",\"origin_at_start\":\"{}\",\"integration\":\"{integration}\"{tail}}}\n", self.base)
    }
}
