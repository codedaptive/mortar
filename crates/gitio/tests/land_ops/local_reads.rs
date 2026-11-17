//! hooks_path, first_parent_list and is_ancestor against real git in throwaway repositories.
use crate::fixture::{commit, repo, tree};
use gitio::{Git, Oid, RunOpts};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

fn canon(path: &Path) -> PathBuf {
    path.canonicalize().unwrap()
}

/// Writes a reference-transaction hook into `dir` that touches `dir/fired`, then moves a ref with plain update-ref:
/// true when git ran the hook from `dir`.
fn git_runs_hooks_from(git: &Git, dir: &Path) -> bool {
    fs::create_dir_all(dir).unwrap();
    let hook = dir.join("reference-transaction");
    fs::write(&hook, format!("#!/bin/sh\ntouch '{}'\n", dir.join("fired").display())).unwrap();
    fs::set_permissions(&hook, fs::Permissions::from_mode(0o755)).unwrap();
    let blob = git.run(&["hash-object", "-w", "--stdin"], &RunOpts { stdin: Some(b"x\n".to_vec()), ..RunOpts::default() });
    let oid = String::from_utf8(blob.unwrap().stdout).unwrap();
    git.run(&["update-ref", "refs/probe/hook", oid.trim_end()], &RunOpts::default()).unwrap();
    dir.join("fired").exists()
}

fn config(git: &Git, key: &str, value: &str) {
    git.run(&["config", key, value], &RunOpts::default()).unwrap();
}

/// A repository with one commit on HEAD's branch and a linked worktree at `<repo>/wt`.
fn with_linked_worktree() -> (crate::support::GitRepo, Git) {
    let r = repo();
    let c = commit(&r.git, &tree(&r.git, &[("f", b"f\n")]), &[], "c\n");
    r.git.run(&["update-ref", "HEAD", c.as_str()], &RunOpts::default()).unwrap();
    r.git.run(&["worktree", "add", "-q", "--detach", "wt", c.as_str()], &RunOpts::default()).unwrap();
    let wt = Git { repo: r.path.join("wt"), ..r.git.clone() };
    (r, wt)
}

#[test]
fn hooks_path_default_is_the_common_dirs_hooks() {
    let r = repo();
    let hooks = r.git.hooks_path().unwrap();
    assert_eq!(hooks, canon(&r.path).join(".git/hooks"));
    assert!(git_runs_hooks_from(&r.git, &hooks));
}

#[test]
fn hooks_path_absolute_core_hooks_path_is_as_set() {
    let r = repo();
    let set = canon(&r.path).join("elsewhere/hooks");
    config(&r.git, "core.hooksPath", set.to_str().unwrap());
    assert_eq!(r.git.hooks_path().unwrap(), set);
    assert!(git_runs_hooks_from(&r.git, &set));
}

#[test]
fn hooks_path_relative_core_hooks_path_is_against_the_worktree_top() {
    let (r, wt) = with_linked_worktree();
    config(&r.git, "core.hooksPath", "rel/h");
    fs::create_dir(r.path.join("sub")).unwrap();
    let sub = Git { repo: r.path.join("sub"), ..r.git.clone() };
    assert_eq!(sub.hooks_path().unwrap(), canon(&r.path).join("rel/h"));
    assert_eq!(wt.hooks_path().unwrap(), canon(&r.path).join("wt/rel/h"));
    assert!(git_runs_hooks_from(&wt, &canon(&r.path).join("wt/rel/h")));
}

#[test]
fn hooks_path_from_a_linked_worktree_is_the_main_repos_hooks() {
    let (r, wt) = with_linked_worktree();
    let hooks = wt.hooks_path().unwrap();
    assert_eq!(hooks, canon(&r.path).join(".git/hooks"));
    assert!(git_runs_hooks_from(&wt, &hooks));
}

#[test]
fn hooks_path_in_a_bare_repo_is_its_own_hooks() {
    let r = repo();
    r.git.run(&["init", "-q", "--bare", "b.git"], &RunOpts::default()).unwrap();
    let bare = Git { repo: r.path.join("b.git"), ..r.git.clone() };
    assert_eq!(bare.hooks_path().unwrap(), canon(&r.path).join("b.git/hooks"));
    config(&bare, "core.hooksPath", "rel");
    assert_eq!(bare.hooks_path().unwrap(), canon(&r.path).join("b.git/rel"));
    assert!(git_runs_hooks_from(&bare, &canon(&r.path).join("b.git/rel")));
}

/// root; main: root, m1; side: root, s1; x merges s1 into m1 (first parent m1); t on x.
struct History { root: Oid, m1: Oid, s1: Oid, x: Oid, t: Oid }

fn history(git: &Git) -> History {
    let at = |text: &[u8], parents: &[Oid], message: &str| commit(git, &tree(git, &[("f", text)]), parents, message);
    let root = at(b"root\n", &[], "root\n");
    let m1 = at(b"m1\n", std::slice::from_ref(&root), "m1\n");
    let s1 = at(b"s1\n", std::slice::from_ref(&root), "s1\n");
    let x = at(b"x\n", &[m1.clone(), s1.clone()], "x\n");
    let t = at(b"t\n", std::slice::from_ref(&x), "t\n");
    History { root, m1, s1, x, t }
}

const UNKNOWN: &str = "1111111111111111111111111111111111111111";

#[test]
fn first_parent_list_over_a_merge_is_first_parents_oldest_first() {
    let r = repo();
    let h = history(&r.git);
    assert_eq!(r.git.first_parent_list(&h.root, &h.t).unwrap(), vec![h.m1, h.x, h.t]);
}

#[test]
fn first_parent_list_is_empty_for_base_equal_tip_or_holding_it() {
    let r = repo();
    let h = history(&r.git);
    assert_eq!(r.git.first_parent_list(&h.t, &h.t).unwrap(), Vec::<Oid>::new());
    assert_eq!(r.git.first_parent_list(&h.t, &h.s1).unwrap(), Vec::<Oid>::new());
}

#[test]
fn first_parent_list_from_a_non_ancestor_is_the_tips_line_it_cannot_reach() {
    let r = repo();
    let h = history(&r.git);
    assert_eq!(r.git.first_parent_list(&h.m1, &h.s1).unwrap(), vec![h.s1]);
}

#[test]
fn first_parent_list_of_an_unknown_commit_is_gits_error() {
    let r = repo();
    let h = history(&r.git);
    let err = r.git.first_parent_list(&Oid::parse(UNKNOWN).unwrap(), &h.t).unwrap_err();
    assert_eq!(err.code, Some(128));
    assert!(err.stderr.starts_with("fatal: Invalid revision range"), "{}", err.stderr);
}

#[test]
fn is_ancestor_true_false_and_true_for_the_same_commit() {
    let r = repo();
    let h = history(&r.git);
    assert_eq!(r.git.is_ancestor(&h.root, &h.t), Ok(true));
    assert_eq!(r.git.is_ancestor(&h.s1, &h.t), Ok(true));
    assert_eq!(r.git.is_ancestor(&h.t, &h.root), Ok(false));
    assert_eq!(r.git.is_ancestor(&h.m1, &h.s1), Ok(false));
    assert_eq!(r.git.is_ancestor(&h.x, &h.x), Ok(true));
}

#[test]
fn is_ancestor_of_an_unknown_commit_is_gits_error_128() {
    let r = repo();
    let h = history(&r.git);
    let err = r.git.is_ancestor(&Oid::parse(UNKNOWN).unwrap(), &h.t).unwrap_err();
    assert_eq!(err.code, Some(128));
    assert_eq!(err.stderr, format!("fatal: Not a valid commit name {UNKNOWN}\n"));
}
