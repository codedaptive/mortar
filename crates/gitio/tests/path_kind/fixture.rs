//! A throwaway repository holding one entry of each kind, built with raw git, never with the method under test.
use crate::support::GitRepo;
use gitio::{Git, Oid, RunOpts};

/// A fresh repository whose tree holds `file` (a blob), `link` (a symlink, mode 120000), `dir/x`, `sub` (a gitlink
/// to a real commit of the same repository), `:colon` and `new\nline` (blobs); the tree's id and a commit of it.
pub fn holding_each_kind() -> (GitRepo, Oid, Oid) {
    let mut r = GitRepo::new();
    r.git.env.extend([
        ("GIT_AUTHOR_NAME".into(), "Fixture Author".into()),
        ("GIT_AUTHOR_EMAIL".into(), "author@example.invalid".into()),
        ("GIT_AUTHOR_DATE".into(), "2000-01-01T00:00:00 +0000".into()),
        ("GIT_COMMITTER_NAME".into(), "Fixture Committer".into()),
        ("GIT_COMMITTER_EMAIL".into(), "committer@example.invalid".into()),
        ("GIT_COMMITTER_DATE".into(), "2000-01-02T00:00:00 +0000".into()),
    ]);
    let git = &r.git;
    let file = blob(git, b"file\n");
    let target = blob(git, b"file");
    let inner = blob(git, b"inner\n");
    let submodule = commit(git, &tree(git, &[("100644", "x", &inner)]));
    let root = tree(
        git,
        &[
            ("100644", "file", &file),
            ("120000", "link", &target),
            ("100644", "dir/x", &inner),
            ("160000", "sub", &submodule),
            ("100644", ":colon", &file),
            ("100644", "new\nline", &file),
        ],
    );
    let head = commit(git, &root);
    (r, root, head)
}

fn blob(git: &Git, bytes: &[u8]) -> Oid {
    oid(git, &["hash-object", "-w", "--stdin"], &RunOpts { stdin: Some(bytes.to_vec()), ..RunOpts::default() })
}

fn commit(git: &Git, tree: &Oid) -> Oid {
    oid(git, &["commit-tree", tree.as_str()], &RunOpts { stdin: Some(b"fixture\n".to_vec()), ..RunOpts::default() })
}

/// A tree of `(mode, path, object)` entries, built in a fixture-only index.
fn tree(git: &Git, entries: &[(&str, &str, &Oid)]) -> Oid {
    let index = git.repo.join(".git/path-kind-fixture-index");
    let opts = RunOpts { env: vec![("GIT_INDEX_FILE".into(), index.into_os_string())], ..RunOpts::default() };
    git.run(&["read-tree", "--empty"], &opts).unwrap();
    for (mode, path, object) in entries {
        git.run(&["update-index", "--add", "--cacheinfo", mode, object.as_str(), path], &opts).unwrap();
    }
    oid(git, &["write-tree"], &opts)
}

fn oid(git: &Git, args: &[&str], opts: &RunOpts) -> Oid {
    Oid::parse(String::from_utf8(git.run(args, opts).unwrap().stdout).unwrap().trim_end()).unwrap()
}
