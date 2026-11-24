//! The landing ZITs' throwaway repository: a base commit on `main` and `integration`, and helpers that build trees,
//! commits and Squash values with plain git, never with land's own code.

use gitio::{Git, Oid, RunOpts};
use land::{LandRefs, Squash};
use oplog::ChangeId;
use std::ffi::OsString;
use std::path::PathBuf;
use testkit::TestRepo;

/// The change ID of the first squash in every case.
pub const ID_A: &str = "klmnopqrstuvwxyzklmnopqrstuvwxyz";
/// The change ID of a second squash, in the cases that land or try to land two.
pub const ID_B: &str = "zyxwvutsrqponmlkzyxwvutsrqponmlk";
/// The committer date of the test's own Git.
pub const DATE: &str = "2000-01-01T00:00:00+00:00";
/// The actor every case's own land and back_out record.
pub const ACTOR: &str = "zit@host";
/// The base commit's only file.
pub const BASE_FILES: [(&str, &str); 1] = [("a.txt", "base\n")];
/// A squash on the base adding s.txt.
pub const SQUASH_FILES: [(&str, &str); 2] = [("a.txt", "base\n"), ("s.txt", "squash\n")];
/// Origin moved past the base, adding o.txt.
pub const ORIGIN_FILES: [(&str, &str); 2] = [("a.txt", "base\n"), ("o.txt", "origin\n")];
/// What a merge of SQUASH_FILES into ORIGIN_FILES holds.
pub const MERGED_FILES: [(&str, &str); 3] = [("a.txt", "base\n"), ("o.txt", "origin\n"), ("s.txt", "squash\n")];

pub struct Fixture {
    pub repo: TestRepo,
    pub git: Git,
    pub common: PathBuf,
    /// The first commit, where `main` and `integration` start.
    pub base: Oid,
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

/// The one-landing form for zone `land`, written out independently of the state writer.
pub fn state_text(id: &str, squash: &Oid, origin: &Oid, integration: &Oid) -> String {
    format!(
        "{{\"change_id\":\"{id}\",\"zone\":\"land\",\"squash\":\"{squash}\",\
         \"origin_at_start\":\"{origin}\",\"integration\":\"{integration}\"}}\n"
    )
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
        let mut git = Git::new(&repo.path);
        git.env = git_env(DATE);
        let base = git.commit_tree(&tree(&git, &BASE_FILES), &[], "base\n", None, None).unwrap();
        let fixture = Fixture { common: repo.path.join(".git"), repo, git, base };
        fixture.set_branch("main", &fixture.base);
        fixture.set_branch("integration", &fixture.base);
        fixture
    }

    pub fn set_branch(&self, name: &str, commit: &Oid) {
        self.git.run(&["update-ref", &format!("refs/heads/{name}"), commit.as_str()], &RunOpts::default()).unwrap();
    }

    pub fn branch(&self, name: &str) -> Oid {
        self.git.read_ref(&format!("refs/heads/{name}")).unwrap().unwrap()
    }

    /// Moves origin past the base: a commit on the base holding `files`, with `main` and `integration` set to it.
    pub fn move_origin(&self, files: &[(&str, &str)]) -> Oid {
        let tree = tree(&self.git, files);
        let moved = self.git.commit_tree(&tree, std::slice::from_ref(&self.base), "origin moved\n", None, None).unwrap();
        self.set_branch("main", &moved);
        self.set_branch("integration", &moved);
        moved
    }

    /// A squash of zone `land` on `parent` holding `files`, its message carrying the Mortar trailers.
    pub fn squash(&self, parent: &Oid, files: &[(&str, &str)], id: &str) -> Squash {
        let message = format!(
            "land: squash of r1\n\nMortar-Change-Id: {id}\nMortar-Zone: land\nMortar-Request: r1\n"
        );
        Squash {
            commit: self.git.commit_tree(&tree(&self.git, files), std::slice::from_ref(parent), &message, None, None).unwrap(),
            change_id: ChangeId::parse(id).unwrap(),
            source: parent.clone(),
            requests: vec!["r1".to_string()],
        }
    }

    /// The text of `<common>/mortar/landing.json`, or None when there is none.
    pub fn landing_json(&self) -> Option<String> {
        std::fs::read_to_string(self.common.join("mortar").join("landing.json")).ok()
    }
}
