//! Throwaway repositories and objects built with raw git, never with a method under test.
use crate::support::GitRepo;
use gitio::{Git, Oid, RunOpts};

/// A fresh repository with fixed author and committer, so fixture commits need no user config.
pub fn repo() -> GitRepo {
    let mut repo = GitRepo::new();
    repo.git.env.extend([
        ("GIT_AUTHOR_NAME".into(), "Fixture Author".into()),
        ("GIT_AUTHOR_EMAIL".into(), "author@example.invalid".into()),
        ("GIT_AUTHOR_DATE".into(), "2000-01-01T00:00:00 +0000".into()),
        ("GIT_COMMITTER_NAME".into(), "Fixture Committer".into()),
        ("GIT_COMMITTER_EMAIL".into(), "committer@example.invalid".into()),
        ("GIT_COMMITTER_DATE".into(), "2000-01-02T00:00:00 +0000".into()),
    ]);
    repo
}

/// The id of `bytes` written as a blob.
pub fn blob(git: &Git, bytes: &[u8]) -> Oid {
    oid(git, &["hash-object", "-w", "--stdin"], &RunOpts { stdin: Some(bytes.to_vec()), ..RunOpts::default() })
}

/// A commit of `tree` with no parents.
pub fn commit(git: &Git, tree: &Oid) -> Oid {
    oid(git, &["commit-tree", tree.as_str()], &RunOpts { stdin: Some(b"fixture\n".to_vec()), ..RunOpts::default() })
}

/// A tree of `(mode, path, object)` entries, built in a fixture-only index.
pub fn tree(git: &Git, entries: &[(&str, &str, &Oid)]) -> Oid {
    let index = git.repo.join(".git/typed-refusals-fixture-index");
    let opts = RunOpts { env: vec![("GIT_INDEX_FILE".into(), index.into_os_string())], ..RunOpts::default() };
    git.run(&["read-tree", "--empty"], &opts).unwrap();
    for (mode, path, object) in entries {
        git.run(&["update-index", "--add", "--cacheinfo", mode, object.as_str(), path], &opts).unwrap();
    }
    oid(git, &["write-tree"], &opts)
}

/// A tree of plain files.
pub fn files(git: &Git, files: &[(&str, &[u8])]) -> Oid {
    let blobs: Vec<(&str, Oid)> = files.iter().map(|(path, bytes)| (*path, blob(git, bytes))).collect();
    let entries: Vec<(&str, &str, &Oid)> = blobs.iter().map(|(path, id)| ("100644", *path, id)).collect();
    tree(git, &entries)
}

fn oid(git: &Git, args: &[&str], opts: &RunOpts) -> Oid {
    Oid::parse(String::from_utf8(git.run(args, opts).unwrap().stdout).unwrap().trim_end()).unwrap()
}
