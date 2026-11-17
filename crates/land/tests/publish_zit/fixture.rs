//! The publish ZITs' throwaway repositories: a local repository with a base commit on `main` and `integration`, and
//! a bare remote `origin` (inside a second TestRepo's directory, removed with it) whose `main` is the base. Trees,
//! commits, squashes and the other clone's push are made with plain git, never with land's own code. A push intent
//! left open is opened by hand through oplog's open_remote_intent, and the push it stands for made with plain git.

use gitio::{Git, Oid, RunOpts};
use land::{land, LandAnswer, LandRefs, Squash};
use oplog::{
    open_remote_intent, read_marked_log, ChangeId, MarkedRecord, Op, OpPhase, OpResult, RefMove, RemoteIntentOutcome,
};
use std::ffi::OsString;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use testkit::TestRepo;

/// What someone else pushes on top of the squash: the squash's files plus o.txt.
pub const PAST_SQUASH_FILES: [(&str, &str); 3] = [("a.txt", "base\n"), ("o.txt", "other\n"), ("s.txt", "squash\n")];
/// The change ID of the squash in every case.
pub const ID_A: &str = "klmnopqrstuvwxyzklmnopqrstuvwxyz";
/// The committer date of the test's own Git.
pub const DATE: &str = "2000-01-01T00:00:00+00:00";
/// The actor every case's own land and publish record.
pub const ACTOR: &str = "zit@host";
/// The base commit's only file.
pub const BASE_FILES: [(&str, &str); 1] = [("a.txt", "base\n")];
/// A squash on the base adding s.txt.
pub const SQUASH_FILES: [(&str, &str); 2] = [("a.txt", "base\n"), ("s.txt", "squash\n")];
/// What someone else pushes onto the remote's main: the base plus o.txt.
pub const OTHER_FILES: [(&str, &str); 2] = [("a.txt", "base\n"), ("o.txt", "other\n")];
/// The squash ref of the zone named in every squash's Mortar-Zone trailer.
pub const SQUASH_REF: &str = "refs/mortar/squash/land";
/// The op id of the push intent a case opens by hand; new_op_id never makes this form.
pub const OPEN_PUSH_OP_ID: &str = "zit-open-push";

pub struct Fixture {
    pub repo: TestRepo,
    pub git: Git,
    pub common: PathBuf,
    /// The first commit, where local `main` and `integration` and the remote's `main` start.
    pub base: Oid,
    /// Holds the bare remote and the other clone; removed with them.
    pub elsewhere: TestRepo,
    /// The bare remote's git.
    pub remote: Git,
}

/// Git's config isolated and a fixed identity and dates, so commit_tree needs no user config.
pub fn git_env(date: &str) -> Vec<(OsString, OsString)> {
    [
        ("GIT_CONFIG_NOSYSTEM", "1"),
        ("GIT_CONFIG_GLOBAL", "/dev/null"),
        ("GIT_AUTHOR_NAME", "Zit"),
        ("GIT_AUTHOR_EMAIL", "zit@example.invalid"),
        ("GIT_AUTHOR_DATE", date),
        ("GIT_COMMITTER_NAME", "Zit"),
        ("GIT_COMMITTER_EMAIL", "zit@example.invalid"),
        ("GIT_COMMITTER_DATE", date),
    ]
    .map(|(key, value)| (key.into(), value.into()))
    .to_vec()
}

pub fn refs() -> LandRefs {
    LandRefs {
        origin_branch: "main".to_string(),
        integration_branch: "integration".to_string(),
        remote: "origin".to_string(),
    }
}

/// landing.json as encode_landing_state lays it out, written by hand.
pub fn state_text(id: &str, squash: &Oid, origin: &Oid, integration: &Oid) -> String {
    format!(
        "{{\"change_id\":\"{id}\",\"zone\":\"land\",\"squash\":\"{squash}\",\"origin_at_start\":\"{origin}\",\
         \"integration\":\"{integration}\"}}\n"
    )
}

fn git_at(path: PathBuf) -> Git {
    let mut git = Git::new(path);
    git.env = git_env(DATE);
    git
}

fn run(git: &Git, args: &[&str]) {
    git.run(args, &RunOpts::default()).unwrap();
}

fn oid_out(git: &Git, args: &[&str], stdin: Vec<u8>) -> Oid {
    let output = git.run(args, &RunOpts { stdin: Some(stdin), ..RunOpts::default() }).unwrap();
    Oid::parse(String::from_utf8(output.stdout).unwrap().trim_end()).unwrap()
}

/// A tree holding exactly `files` (name, content), each at the top level.
pub fn tree(git: &Git, files: &[(&str, &str)]) -> Oid {
    let mut listing = String::new();
    for (name, content) in files {
        let blob = oid_out(git, &["hash-object", "-w", "--stdin"], content.as_bytes().to_vec());
        listing.push_str(&format!("100644 blob {blob}\t{name}\n"));
    }
    oid_out(git, &["mktree"], listing.into_bytes())
}

impl Fixture {
    pub fn new() -> Self {
        let repo = TestRepo::new().unwrap();
        let git = git_at(repo.path.clone());
        let base = git.commit_tree(&tree(&git, &BASE_FILES), &[], "base\n", None, None).unwrap();
        let elsewhere = TestRepo::new().unwrap();
        let bare = elsewhere.path.join("remote.git");
        run(&git_at(elsewhere.path.clone()), &["init", "--bare", "--object-format=sha1", bare.to_str().unwrap()]);
        let remote = git_at(bare.clone());
        let fixture = Fixture { common: repo.path.join(".git"), repo, git, base, elsewhere, remote };
        fixture.set_ref("refs/heads/main", &fixture.base);
        fixture.set_ref("refs/heads/integration", &fixture.base);
        run(&fixture.git, &["remote", "add", "origin", bare.to_str().unwrap()]);
        run(&fixture.git, &["push", "origin", "refs/heads/main:refs/heads/main"]);
        fixture
    }

    /// A fixture whose squash of zone `land` on the base was landed by land (a fast-forward of integration) and
    /// whose squash ref holds it, as the squash step leaves it.
    pub fn landed() -> (Self, Squash) {
        let fx = Fixture::new();
        let squash = fx.squash(&fx.base, &SQUASH_FILES);
        fx.set_ref(SQUASH_REF, &squash.commit);
        assert_eq!(
            land(&fx.git, &refs(), &squash, ACTOR).unwrap(),
            LandAnswer::Landed { integration: squash.commit.clone() }
        );
        (fx, squash)
    }

    pub fn set_ref(&self, refname: &str, commit: &Oid) {
        run(&self.git, &["update-ref", refname, commit.as_str()]);
    }

    pub fn branch(&self, name: &str) -> Oid {
        self.git.read_ref(&format!("refs/heads/{name}")).unwrap().unwrap()
    }

    /// The remote's `main`, read in the bare repository itself.
    pub fn remote_main(&self) -> Option<Oid> {
        self.remote.read_ref("refs/heads/main").unwrap()
    }

    /// A squash of zone `land` on `parent` holding `files`, its message carrying the Mortar trailers.
    pub fn squash(&self, parent: &Oid, files: &[(&str, &str)]) -> Squash {
        let message = format!(
            "land: squash of WO-land-4\n\nMortar-Change-Id: {ID_A}\nMortar-Zone: land\nMortar-Request: WO-land-4\n"
        );
        Squash {
            commit: self.git.commit_tree(&tree(&self.git, files), std::slice::from_ref(parent), &message, None, None).unwrap(),
            change_id: ChangeId::parse(ID_A).unwrap(),
            source: parent.clone(),
            requests: vec!["WO-land-4".to_string()],
        }
    }

    /// Someone else: a second clone of the remote commits OTHER_FILES on the base and pushes it onto the remote's
    /// `branch`. Returns that commit, X.
    pub fn someone_else_pushes(&self, branch: &str) -> Oid {
        self.someone_else_pushes_on(&self.base, &OTHER_FILES, branch)
    }

    /// Someone else: a second clone of the remote commits `files` on `parent` (already on the remote) and pushes it
    /// onto the remote's `branch`. Returns that commit.
    pub fn someone_else_pushes_on(&self, parent: &Oid, files: &[(&str, &str)], branch: &str) -> Oid {
        let clone = self.elsewhere.path.join("other");
        let bare = self.remote.repo.to_str().unwrap();
        run(&git_at(self.elsewhere.path.clone()), &["clone", "--quiet", bare, clone.to_str().unwrap()]);
        let other = git_at(clone);
        let tree = tree(&other, files);
        let moved = other.commit_tree(&tree, std::slice::from_ref(parent), "other\n", None, None).unwrap();
        run(&other, &["push", "origin", &format!("{moved}:refs/heads/{branch}")]);
        moved
    }

    /// Installs `script` as the bare remote's executable pre-receive hook.
    pub fn install_pre_receive(&self, script: &str) {
        let hook = self.remote.repo.join("hooks").join("pre-receive");
        std::fs::write(&hook, script).unwrap();
        std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    /// The text of `<common>/mortar/landing.json`, or None when there is none.
    pub fn landing_json(&self) -> Option<String> {
        std::fs::read_to_string(self.common.join("mortar").join("landing.json")).ok()
    }

    /// Opens publish's push intent for the move of the remote's main from the base to `new`, marked with the remote
    /// `origin`, as a publish killed after opening it leaves it.
    pub fn open_push_intent(&self, new: &Oid) {
        let op = Op {
            op_id: OPEN_PUSH_OP_ID.to_string(),
            // Duplicates publish_push_op's key form (pub(crate) in land): the key a later publish looks for.
            key: format!("publish-push:origin:refs/heads/main:{}:{new}", self.base),
            kind: "publish-push".to_string(),
            actor: ACTOR.to_string(),
            reason: "publish-push opened by the fixture".to_string(),
            at: 946_684_800,
            moves: vec![RefMove {
                refname: "refs/heads/main".to_string(),
                old: Some(self.base.clone()),
                new: new.clone(),
            }],
        };
        let outcome = open_remote_intent(&self.git, &self.common, &op, "origin").unwrap();
        assert!(matches!(outcome, RemoteIntentOutcome::Opened(_)), "{outcome:?}");
    }

    /// Pushes `commit` from the local repository onto the remote's main with plain git, outside any op.
    pub fn push_remote_main(&self, commit: &Oid) {
        run(&self.git, &["push", "origin", &format!("{commit}:refs/heads/main")]);
    }

    /// The op log's publish-push records, in log order, with their remote marks.
    pub fn push_records(&self) -> Vec<MarkedRecord> {
        read_marked_log(&self.common)
            .unwrap()
            .records
            .into_iter()
            .filter(|marked| marked.record.kind == "publish-push")
            .collect()
    }

    /// The op log's publish-push records in log order: phase, result and remote mark.
    pub fn push_log(&self) -> Vec<(OpPhase, Option<OpResult>, Option<String>)> {
        self.push_records()
            .into_iter()
            .map(|marked| (marked.record.phase, marked.record.result, marked.remote))
            .collect()
    }
}
