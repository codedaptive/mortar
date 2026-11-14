//! The squash ZITs' throwaway repository: base B (files `base` and `f`) on `main` and `integration`, and helpers
//! that build commits and read refs and messages with plain git, never with land's own code.

use gitio::{Git, Oid};
use land::{LandRefs, Squash, SquashAnswer};
use std::ffi::OsString;
use std::path::Path;
use std::process::Command;
use testkit::TestRepo;

/// The zone every case squashes; its branch is `refs/heads/zone/land`.
pub const ZONE: &str = "land";
/// The ref land keeps the zone's squash at (S-57).
pub const SQUASH_REF: &str = "refs/mortar/squash/land";
/// The actor every case's squash records.
pub const ACTOR: &str = "zit@host";

/// Git's config isolated (#1416) and a fixed identity and dates, so commits need no user config and every run
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

pub fn oid(text: &str) -> Oid {
    Oid::parse(text).unwrap()
}

/// The Squash of a Made answer; any other answer fails the test.
pub fn made(answer: SquashAnswer) -> Squash {
    match answer {
        SquashAnswer::Made(squash) => squash,
        other => panic!("expected Made, got {other:?}"),
    }
}

/// A commit message split at its last paragraph: the text before it (ending in its newline) and its lines.
pub fn split_trailers(message: &str) -> (String, Vec<String>) {
    let at = message.trim_end().rfind("\n\n").unwrap();
    (message[..at + 1].to_string(), message[at + 2..].lines().map(str::to_string).collect())
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
        for (name, content) in [("base", "base\n"), ("f", "f\n")] {
            std::fs::write(fixture.path().join(name), content).unwrap();
            fixture.git(&["add", name]);
        }
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

    /// A commit on `parent` writing each (name, content), with `subject` and, when given, a Mortar-Request trailer.
    pub fn commit_on(&self, parent: &str, files: &[(&str, &str)], subject: &str, request: Option<&str>) -> String {
        self.git(&["checkout", "-q", "--detach", parent]);
        for (name, content) in files {
            std::fs::write(self.path().join(name), content).unwrap();
            self.git(&["add", name]);
        }
        match request {
            Some(request) => self.git(&["commit", "-q", "-m", subject, "-m", &format!("Mortar-Request: {request}")]),
            None => self.git(&["commit", "-q", "-m", subject]),
        };
        self.git(&["rev-parse", "HEAD"])
    }

    /// Points `refs/heads/<name>` at `commit`.
    pub fn set_branch(&self, name: &str, commit: &str) {
        self.git(&["update-ref", &format!("refs/heads/{name}"), commit]);
    }

    /// The commit `refname` names, or None when it is absent.
    pub fn ref_of(&self, refname: &str) -> Option<String> {
        let found = self.git(&["for-each-ref", "--format=%(objectname)", refname]);
        (!found.is_empty()).then_some(found)
    }

    /// The message of `commit`, byte for byte.
    pub fn message(&self, commit: &str) -> String {
        let object = self.raw(&["cat-file", "commit", commit]);
        object.split_once("\n\n").unwrap().1.to_string()
    }

    /// The bytes of the op log, or None when there is none.
    pub fn oplog(&self) -> Option<Vec<u8>> {
        std::fs::read(self.path().join(".git").join("mortar").join("oplog.jsonl")).ok()
    }
}
