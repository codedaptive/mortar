//! worktree_fill against real git: a worktree added empty, in a repository whose shared config turns sparse
//! checkout on and whose main worktree is sparse.
use crate::fixture::{commit, repo, set_ref, tree};
use gitio::{Git, RunOpts};
use std::fs;
use std::path::Path;

fn read(git: &Git, args: &[&str]) -> String {
    let out = git.run(args, &RunOpts { read_only: true, ..RunOpts::default() }).unwrap();
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn worktree_fill_writes_every_file_with_shared_sparse_checkout_and_leaves_status_empty() {
    let r = repo();
    let files: &[(&str, &[u8])] = &[("top.txt", b"top\n"), ("d1/a", b"in the cone\n"), ("d2/deep/b", b"outside\n")];
    let c = commit(&r.git, &tree(&r.git, files), &[], "c\n");
    r.git.run(&["symbolic-ref", "HEAD", "refs/heads/main"], &RunOpts::default()).unwrap();
    set_ref(&r.git, "refs/heads/main", &c);
    r.git.run(&["sparse-checkout", "set", "--cone", "d1"], &RunOpts::default()).unwrap();
    r.git.run(&["config", "core.sparseCheckout", "true"], &RunOpts::default()).unwrap();
    let shared = r.path.join(".git/config");
    assert_eq!(read(&r.git, &["config", "--file", shared.to_str().unwrap(), "core.sparseCheckout"]), "true\n");
    assert_eq!(read(&r.git, &["sparse-checkout", "list"]), "d1\n");

    let folder = r.path.join("probe");
    r.git.worktree_add(&folder, &c, None).unwrap();
    assert_eq!(fs::read_dir(&folder).unwrap().count(), 1, "worktree_add leaves only .git");
    r.git.worktree_fill(&folder).unwrap();

    assert_eq!(fs::read(folder.join("top.txt")).unwrap(), b"top\n");
    assert_eq!(fs::read(folder.join("d1/a")).unwrap(), b"in the cone\n");
    assert_eq!(fs::read(folder.join("d2/deep/b")).unwrap(), b"outside\n");
    let probe = Git { repo: folder, ..r.git.clone() };
    assert_eq!(read(&probe, &["status", "--porcelain"]), "");
    assert_eq!(read(&probe, &["ls-files", "-t"]), "H d1/a\nH d2/deep/b\nH top.txt\n");
}

#[test]
fn worktree_fill_refuses_an_empty_path_before_running_git() {
    let r = repo();
    let err = r.git.worktree_fill(Path::new("")).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert_eq!(err.stderr, "not a usable worktree path (empty): \"\"");
}
