//! The withdraw ZITs' throwaway repository: base B (file `base`) on `main` and `integration`, and helpers. The
//! zone's commits and refs are built, and refs and the op log read, with plain git and the file system, never with
//! land's own readers; the squash itself is land::squash's.

use gitio::Git;
use land::{squash, LandRefs, Squash, SquashAnswer};
use std::ffi::OsString;
use std::path::Path;
use std::process::Command;
use testkit::TestRepo;

/// The zone every case squashes and withdraws; its branch is `refs/heads/zone/land`.
pub const ZONE: &str = "land";
/// The ref land keeps the zone's squash at, `refs/mortar/squash/<zone>` for this zone.
pub const SQUASH_REF: &str = "refs/mortar/squash/land";
/// The actor every case's squash, landing and withdraw records.
pub const ACTOR: &str = "zit@host";

/// Git's config isolated and a fixed identity and dates, so commits need no user config and every run
/// builds the same ones.
fn git_env() -> Vec<(OsString, OsString)> {
    [
        ("GIT_CONFIG_GLOBAL", "/dev/null"),
        ("GIT_CONFIG_NOSYSTEM", "1"),
        ("GIT_AUTHOR_NAME", "Fixture"),
        ("GIT_AUTHOR_EMAIL", "fixture@example.invalid"),
        ("GIT_AUTHOR_DATE", "2026-01-01T00:00:00Z"),
        ("GIT_COMMITTER_NAME", "Fixture"),
        ("GIT_COMMITTER_EMAIL", "fixture@example.invalid"),
        ("GIT_COMMITTER_DATE", "2026-01-01T00:00:00Z"),
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

/// The Squash of a Made answer; any other answer fails the test.
pub fn made(answer: SquashAnswer) -> Squash {
    match answer {
        SquashAnswer::Made(squash) => squash,
        other => panic!("expected Made, got {other:?}"),
    }
}

pub struct Fixture {
    pub repo: TestRepo,
    /// Base commit B, where `main` and `integration` start.
    pub base: String,
}

impl Fixture {
    pub fn new() -> Self {
        let repo = TestRepo::new().unwrap();
        let fixture = Fixture { repo, base: String::new() };
        std::fs::write(fixture.path().join("base"), "base\n").unwrap();
        fixture.git(&["add", "base"]);
        fixture.git(&["commit", "-q", "-m", "base"]);
        let base = fixture.git(&["rev-parse", "HEAD"]);
        fixture.set_branch("integration", &base);
        Fixture { base, ..fixture }
    }

    fn path(&self) -> &Path {
        &self.repo.path
    }

    /// The gitio value handed to land: config isolated and the fixed identity, for squash's commit_tree.
    pub fn land_git(&self) -> Git {
        Git { program: "git".into(), repo: self.path().to_path_buf(), env: git_env() }
    }

    /// Runs git in the repository with the fixture's environment; its stdout as given.
    pub fn raw(&self, args: &[&str]) -> String {
        let mut command = Command::new("git");
        command.arg("-C").arg(self.path()).args(args).envs(git_env());
        // The fixture must target this repository even when the test runs inside a git hook.
        for name in ["GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_PREFIX", "GIT_OBJECT_DIRECTORY"] {
            command.env_remove(name);
        }
        let output = command.output().unwrap();
        assert!(output.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&output.stderr));
        String::from_utf8(output.stdout).unwrap()
    }

    /// Runs git as `raw` does; its stdout trimmed.
    pub fn git(&self, args: &[&str]) -> String {
        self.raw(args).trim().to_string()
    }

    /// A commit on `parent` writing each (name, content), with `subject` and a Mortar-Request trailer `request`.
    pub fn commit_on(&self, parent: &str, files: &[(&str, &str)], subject: &str, request: &str) -> String {
        self.git(&["checkout", "-q", "--detach", parent]);
        for (name, content) in files {
            std::fs::write(self.path().join(name), content).unwrap();
            self.git(&["add", name]);
        }
        self.git(&["commit", "-q", "-m", subject, "-m", &format!("Mortar-Request: {request}")]);
        self.git(&["rev-parse", "HEAD"])
    }

    /// Points `refs/heads/<name>` at `commit`.
    pub fn set_branch(&self, name: &str, commit: &str) {
        self.git(&["update-ref", &format!("refs/heads/{name}"), commit]);
    }

    /// One zone commit on B at `refs/heads/zone/land`, squashed with land::squash: the Made squash.
    pub fn squashed(&self) -> Squash {
        let zone_tip = self.commit_on(&self.base, &[("z", "zone\n")], "add z", "r1");
        self.set_branch("zone/land", &zone_tip);
        made(squash(&self.land_git(), &refs(), ZONE, ACTOR).unwrap())
    }

    /// The commit `refname` names, or None when it is absent.
    pub fn ref_of(&self, refname: &str) -> Option<String> {
        let found = self.git(&["for-each-ref", "--format=%(objectname)", refname]);
        (!found.is_empty()).then_some(found)
    }

    /// Every ref in the repository, one `<refname> <commit>` line each, in git's order.
    pub fn ref_lines(&self) -> Vec<String> {
        self.raw(&["for-each-ref", "--format=%(refname) %(objectname)"]).lines().map(str::to_string).collect()
    }

    /// The bytes of the op log, or None when there is none.
    pub fn oplog(&self) -> Option<Vec<u8>> {
        std::fs::read(self.path().join(".git").join("mortar").join("oplog.jsonl")).ok()
    }
}
