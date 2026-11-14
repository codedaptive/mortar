//! Throwaway repositories, commits, merges and expected trees built with raw git, never with Git::index_tree.
use crate::support::GitRepo;
use gitio::{Git, Oid, RunOpts};
use std::fs;
use std::path::{Path, PathBuf};
use testkit::write_git_shim;

/// A fresh repository on main with fixed author and committer, so fixture commits need no user config.
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

/// A repository whose main has one commit of `files`, checked out with a matching index.
pub fn committed(files: &[(&str, &[u8])]) -> GitRepo {
    let r = repo();
    stage(&r.git, &r.path, files);
    raw(&r.git, &r.path, &["commit", "--quiet", "-m", "base"]);
    r
}

/// Writes `files` into `folder` and stages exactly those paths with `git add`.
pub fn stage(git: &Git, folder: &Path, files: &[(&str, &[u8])]) {
    for (path, bytes) in files {
        let file = folder.join(path);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, bytes).unwrap();
        raw(git, folder, &["add", "--", path]);
    }
}

/// Runs raw git in `folder` and returns its stdout; any non-zero exit panics.
pub fn raw(git: &Git, folder: &Path, args: &[&str]) -> Vec<u8> {
    git.run(args, &RunOpts { cwd: Some(folder.to_path_buf()), ..RunOpts::default() }).unwrap().stdout
}

/// The tree of the commit `folder`'s HEAD names, by `rev-parse HEAD^{tree}`.
pub fn head_tree(git: &Git, folder: &Path) -> Oid {
    parse_oid(&raw(git, folder, &["rev-parse", "HEAD^{tree}"]))
}

/// A tree of plain files, built in a fixture-only index so no worktree's index is touched.
pub fn tree_of(git: &Git, files: &[(&str, &[u8])]) -> Oid {
    let index = git.repo.join(".git/index-tree-fixture-index");
    let opts = RunOpts { env: vec![("GIT_INDEX_FILE".into(), index.into_os_string())], ..RunOpts::default() };
    git.run(&["read-tree", "--empty"], &opts).unwrap();
    for (path, bytes) in files {
        let blob = RunOpts { stdin: Some(bytes.to_vec()), ..RunOpts::default() };
        let id = parse_oid(&git.run(&["hash-object", "-w", "--stdin"], &blob).unwrap().stdout);
        git.run(&["update-index", "--add", "--cacheinfo", "100644", id.as_str(), path], &opts).unwrap();
    }
    parse_oid(&git.run(&["write-tree"], &opts).unwrap().stdout)
}

/// A repository mid-merge: main and `other` both change `a.txt`, `dir/sp ace.txt` and `z.txt` (conflicts), and only
/// `other` changes `clean.txt` and `dir/clean.txt` (merged cleanly, so not unmerged).
pub fn conflicted() -> GitRepo {
    let r = committed(&[
        ("a.txt", b"base\n"), ("clean.txt", b"base\n"), ("dir/clean.txt", b"base\n"),
        ("dir/sp ace.txt", b"base\n"), ("z.txt", b"base\n"),
    ]);
    raw(&r.git, &r.path, &["checkout", "--quiet", "-b", "other"]);
    stage(&r.git, &r.path, &[
        ("a.txt", b"other\n"), ("clean.txt", b"other\n"), ("dir/clean.txt", b"other\n"),
        ("dir/sp ace.txt", b"other\n"), ("z.txt", b"other\n"),
    ]);
    raw(&r.git, &r.path, &["commit", "--quiet", "-m", "other"]);
    raw(&r.git, &r.path, &["checkout", "--quiet", "main"]);
    stage(&r.git, &r.path, &[("a.txt", b"main\n"), ("dir/sp ace.txt", b"main\n"), ("z.txt", b"main\n")]);
    raw(&r.git, &r.path, &["commit", "--quiet", "-m", "main"]);
    let merge = RunOpts { cwd: Some(r.path.clone()), answer_codes: vec![1], ..RunOpts::default() };
    let merged = r.git.run(&["merge", "--no-edit", "other"], &merge).unwrap();
    assert_eq!(merged.status, 1, "the fixture merge must stop on its conflicts");
    r
}

/// Parses one id line of git's stdout.
pub fn parse_oid(bytes: &[u8]) -> Oid {
    Oid::parse(std::str::from_utf8(bytes).unwrap().trim_end()).unwrap()
}

/// A copy of `r.git` whose program is a shim in front of git that records the GIT_INDEX_FILE git was given in the
/// returned trace file. Guarded: a run with no GIT_INDEX_FILE (the index path lookup) leaves the trace as it is.
pub fn traced(r: &GitRepo) -> (Git, PathBuf) {
    let shim = r.path.join(".git/index-tree-trace.sh");
    let trace = r.path.join(".git/index-tree-trace.index");
    write_git_shim(&shim, br#"#!/bin/sh
set -eu
if [ -n "${GIT_INDEX_FILE-}" ]; then printf '%s' "$GIT_INDEX_FILE" > "$MORTAR_TEST_INDEX_TRACE"; fi
exec git "$@"
"#).unwrap();
    let mut git = r.git.clone();
    git.program = shim;
    git.env.push(("MORTAR_TEST_INDEX_TRACE".into(), trace.clone().into_os_string()));
    (git, trace)
}

/// The temporary index git was given, by `trace`, exists no more, nor its lock.
pub fn assert_no_temporary_index(trace: &Path) {
    assert!(trace.exists(), "git was never given a GIT_INDEX_FILE: index_tree read no temporary index");
    let index = PathBuf::from(fs::read_to_string(trace).unwrap());
    assert!(!index.as_os_str().is_empty());
    assert!(!index.exists(), "temporary index survived: {}", index.display());
    assert!(!PathBuf::from(format!("{}.lock", index.display())).exists());
}
