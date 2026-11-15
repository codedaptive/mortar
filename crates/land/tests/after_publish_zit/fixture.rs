//! The after_publish ZITs' throwaway repositories: a local repository with a base commit on `main` and
//! `integration`, and a bare remote `origin` (inside a second TestRepo's directory, removed with it) whose `main` is
//! the base. Zone commits are made with plain git; the publish itself is land's own squash, land and publish, so
//! after_publish reads the trailers squash wrote. made, land_and_publish and origin_untouched are the cases' shared
//! steps for a kept squash published later.

use gitio::{Git, Oid, RunOpts};
use join::OpenRequest;
use land::{land, publish, squash, LandAnswer, LandRefs, PublishAnswer, Squash, SquashAnswer, ZoneRequests};
use oplog::{read_op_log, OpRecord};
use std::ffi::OsString;
use std::path::PathBuf;
use testkit::TestRepo;

/// The zone whose squash every case publishes.
pub const PUBLISHED: &str = "land";
/// The actor every case's own squash, land, publish and after_publish record.
pub const ACTOR: &str = "zit@host";
/// The base commit's files: a.txt holds three lines the zones edit.
pub const BASE_FILES: [(&str, &str); 1] = [("a.txt", "one\ntwo\nthree\n")];
/// The published zone's one commit on the base: a.txt's middle line changed, p.txt added.
pub const PUBLISHED_FILES: [(&str, &str); 2] = [("a.txt", "one\nTWO\nthree\n"), ("p.txt", "published\n")];
/// Zone b's commit on the base: b.txt added, a.txt untouched.
pub const B_FILES: [(&str, &str); 2] = [("a.txt", "one\ntwo\nthree\n"), ("b.txt", "b\n")];
/// Zone c's commit on the base: the same middle line of a.txt the published zone changed, changed otherwise.
pub const C_FILES: [(&str, &str); 1] = [("a.txt", "one\ntwo-c\nthree\n")];

pub struct Fixture {
    /// The local repository's directory, removed when the fixture drops.
    _repo: TestRepo,
    pub git: Git,
    pub common: PathBuf,
    /// The first commit, where local `main` and `integration` and the remote's `main` start.
    pub base: Oid,
    /// Holds the bare remote; removed when the fixture drops.
    _elsewhere: TestRepo,
    /// The bare remote's git.
    pub remote: Git,
}

/// Git's config isolated (#1416) and a fixed identity and dates, so commit_tree needs no user config.
fn git_env() -> Vec<(OsString, OsString)> {
    [
        ("GIT_CONFIG_NOSYSTEM", "1"),
        ("GIT_CONFIG_GLOBAL", "/dev/null"),
        ("GIT_AUTHOR_NAME", "Zit"),
        ("GIT_AUTHOR_EMAIL", "zit@example.invalid"),
        ("GIT_AUTHOR_DATE", "2000-01-01T00:00:00+00:00"),
        ("GIT_COMMITTER_NAME", "Zit"),
        ("GIT_COMMITTER_EMAIL", "zit@example.invalid"),
        ("GIT_COMMITTER_DATE", "2000-01-01T00:00:00+00:00"),
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

/// A zone with the open requests the caller hands in.
pub fn zone(name: &str, requests: Vec<OpenRequest>) -> ZoneRequests {
    ZoneRequests { zone: name.to_string(), requests }
}

/// An open request cut from `base`, its tip `tip`.
pub fn request(name: &str, base: &Oid, tip: &Oid) -> OpenRequest {
    OpenRequest { name: name.to_string(), base: base.clone(), tip: tip.clone() }
}

/// The zone's squash, made with land's own squash.
pub fn made(fx: &Fixture, zone: &str) -> Squash {
    let SquashAnswer::Made(made) = squash(&fx.git, &refs(), zone, ACTOR).unwrap() else {
        panic!("zone {zone} must squash");
    };
    made
}

/// The squash landed and published with land's own code; the origin publish answers.
pub fn land_and_publish(fx: &Fixture, made: &Squash) -> Oid {
    let LandAnswer::Landed { integration } = land(&fx.git, &refs(), made, ACTOR).unwrap() else {
        panic!("the squash must land");
    };
    assert_eq!(publish(&fx.git, &refs(), ACTOR).unwrap(), PublishAnswer::Published { origin: integration.clone() });
    integration
}

/// main and the remote's main are both `origin`.
pub fn origin_untouched(fx: &Fixture, origin: &Oid) {
    assert_eq!(fx.branch("main"), *origin);
    assert_eq!(fx.remote_main(), Some(origin.clone()));
}

fn git_at(path: PathBuf) -> Git {
    let mut git = Git::new(path);
    git.env = git_env();
    git
}

fn run(git: &Git, args: &[&str]) -> Vec<u8> {
    git.run(args, &RunOpts::default()).unwrap().stdout
}

fn oid_out(git: &Git, args: &[&str], stdin: Vec<u8>) -> Oid {
    let output = git.run(args, &RunOpts { stdin: Some(stdin), ..RunOpts::default() }).unwrap();
    Oid::parse(String::from_utf8(output.stdout).unwrap().trim_end()).unwrap()
}

/// A tree holding exactly `files` (name, content), each at the top level.
fn tree(git: &Git, files: &[(&str, &str)]) -> Oid {
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
        let fixture = Fixture { common: repo.path.join(".git"), _repo: repo, git, base, _elsewhere: elsewhere, remote };
        fixture.set_ref("refs/heads/main", &fixture.base);
        fixture.set_ref("refs/heads/integration", &fixture.base);
        run(&fixture.git, &["remote", "add", "origin", bare.to_str().unwrap()]);
        run(&fixture.git, &["push", "origin", "refs/heads/main:refs/heads/main"]);
        fixture
    }

    pub fn set_ref(&self, refname: &str, commit: &Oid) {
        run(&self.git, &["update-ref", refname, commit.as_str()]);
    }

    /// A commit on `parent` whose tree holds exactly `files`, its message `subject`; the zone branch `zone` set to it.
    pub fn join_onto(&self, zone: &str, parent: &Oid, files: &[(&str, &str)], subject: &str) -> Oid {
        let message = format!("{subject}\n");
        let commit = self.git.commit_tree(&tree(&self.git, files), &[parent.clone()], &message, None, None).unwrap();
        self.set_ref(&format!("refs/heads/zone/{zone}"), &commit);
        commit
    }

    /// The tip of `refs/heads/<name>`.
    pub fn branch(&self, name: &str) -> Oid {
        self.git.read_ref(&format!("refs/heads/{name}")).unwrap().unwrap()
    }

    /// The remote's `main`, read in the bare repository itself.
    pub fn remote_main(&self) -> Option<Oid> {
        self.remote.read_ref("refs/heads/main").unwrap()
    }

    /// The published zone squashed, landed and published with land's own code; the published origin commit.
    pub fn publish(&self) -> Oid {
        let SquashAnswer::Made(made) = squash(&self.git, &refs(), PUBLISHED, ACTOR).unwrap() else {
            panic!("the published zone must squash");
        };
        let landed = land(&self.git, &refs(), &made, ACTOR).unwrap();
        assert_eq!(landed, LandAnswer::Landed { integration: made.commit.clone() });
        let published = publish(&self.git, &refs(), ACTOR).unwrap();
        assert_eq!(published, PublishAnswer::Published { origin: made.commit.clone() });
        made.commit
    }

    /// The content of `name` in `commit`'s tree.
    pub fn file(&self, commit: &Oid, name: &str) -> String {
        String::from_utf8(run(&self.git, &["cat-file", "blob", &format!("{}:{name}", commit.as_str())])).unwrap()
    }

    /// The names of the files in `commit`'s tree, in git's order.
    pub fn files(&self, commit: &Oid) -> Vec<String> {
        let listing = String::from_utf8(run(&self.git, &["ls-tree", "--name-only", commit.as_str()])).unwrap();
        listing.lines().map(str::to_string).collect()
    }

    /// The op log's after-publish records, in log order.
    pub fn after_publish_records(&self) -> Vec<OpRecord> {
        let records = read_op_log(&self.common).unwrap().records;
        records.into_iter().filter(|record| record.kind == "after-publish").collect()
    }
}
