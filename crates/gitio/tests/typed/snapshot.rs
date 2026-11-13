use crate::fixture::{blob, commit, repo, set_ref, tree};
use crate::support::TestRepo;
use gitio::{Git, Oid, RunOpts};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

#[test]
fn snapshot_saves_included_changes_without_touching_index_or_leaving_temp_index() {
    // Joins Git::snapshot_tree -> real temporary-index read/add/write -> immutable real index.
    let r = repo();
    let base_tree = tree(&r.git, &[("tracked", b"old\n"), ("excluded/tracked", b"excluded old\n")]);
    let base = commit(&r.git, &base_tree, &[], "base\n");
    set_ref(&r.git, "refs/heads/main", &base);
    r.git.run(&["checkout", "--force", "main"], &RunOpts::default()).unwrap();
    fs::write(r.path.join("tracked"), b"new\n").unwrap();
    fs::write(r.path.join("included"), b"new file\n").unwrap();
    fs::write(r.path.join("excluded/tracked"), b"excluded changed\n").unwrap();
    fs::write(r.path.join("excluded/new"), b"must not enter tree\n").unwrap();
    let index = fs::read(r.path.join(".git/index")).unwrap();
    let (git, trace) = traced_index(&r);
    let snapshot = git.snapshot_tree(&base, &["excluded".into()]).unwrap();
    assert_eq!(blob(&r.git, &snapshot, "tracked"), b"new\n");
    assert_eq!(blob(&r.git, &snapshot, "included"), b"new file\n");
    assert_eq!(blob(&r.git, &snapshot, "excluded/tracked"), b"excluded old\n");
    assert!(r.git.run(&["show", &format!("{snapshot}:excluded/new")], &RunOpts::default()).is_err());
    assert_eq!(fs::read(r.path.join(".git/index")).unwrap(), index);
    assert_eq!(r.git.rev_parse("HEAD").unwrap(), Some(base));
    let temporary = PathBuf::from(fs::read_to_string(trace).unwrap());
    assert!(!temporary.as_os_str().is_empty());
    assert!(!temporary.exists(), "snapshot left {} behind", temporary.display());
}

#[test]
fn sparse_snapshot_keeps_out_of_cone_files_and_real_index_byte_equal() {
    // Joins worktree_add -> sparse_set -> snapshot_tree -> real tree/index observation.
    let r = repo();
    let base_tree = tree(&r.git, &[("root.txt", b"root\n"), ("keep/edited", b"old\n"), ("outside/hidden", b"keep hidden\n")]);
    let base = commit(&r.git, &base_tree, &[], "base\n");
    set_ref(&r.git, "refs/heads/main", &base);
    let folder = r.path.join("sparse-folder");
    r.git.worktree_add(&folder, &base, None).unwrap();
    r.git.sparse_set(&folder, &["keep".into()]).unwrap();
    assert!(!folder.join("outside").exists());
    let git = Git { repo: folder.clone(), ..r.git.clone() };
    let index_path = git.run(&["rev-parse", "--path-format=absolute", "--git-path", "index"], &RunOpts::default()).unwrap();
    let index_path = PathBuf::from(String::from_utf8(index_path.stdout).unwrap().trim_end());
    let original_index = fs::read(&index_path).unwrap();
    fs::write(folder.join("keep/edited"), b"changed\n").unwrap();
    fs::write(folder.join("keep/new"), b"new\n").unwrap();
    fs::write(folder.join("keep/omit"), b"excluded\n").unwrap();
    let snapshot = git.snapshot_tree(&base, &["keep/omit".into()]).unwrap();
    assert_eq!(blob(&git, &snapshot, "outside/hidden"), b"keep hidden\n");
    assert_eq!(blob(&git, &snapshot, "keep/edited"), b"changed\n");
    assert_eq!(blob(&git, &snapshot, "keep/new"), b"new\n");
    assert_eq!(blob(&git, &snapshot, "root.txt"), b"root\n");
    let names = git.run(&["ls-tree", "-r", "--name-only", "-z", snapshot.as_str()], &RunOpts::default()).unwrap();
    assert_eq!(names.stdout, b"keep/edited\0keep/new\0outside/hidden\0root.txt\0");
    assert_eq!(fs::read(index_path).unwrap(), original_index);
    assert!(!folder.join("outside").exists());
    assert_eq!(git.rev_parse("HEAD").unwrap(), Some(base));
}

#[test]
fn failed_snapshot_removes_its_temp_index_and_preserves_the_real_index() {
    // Joins snapshot_tree error return -> temporary-index drop cleanup -> real index persistence.
    let r = repo();
    let contents = tree(&r.git, &[("file", b"base\n")]);
    let base = commit(&r.git, &contents, &[], "base\n");
    set_ref(&r.git, "refs/heads/main", &base);
    r.git.run(&["checkout", "--force", "main"], &RunOpts::default()).unwrap();
    let before = fs::read(r.path.join(".git/index")).unwrap();
    let (git, trace) = traced_index(&r);
    let missing = Oid::parse(&"0".repeat(40)).unwrap();
    assert!(git.snapshot_tree(&missing, &[]).is_err());
    let temporary = PathBuf::from(fs::read_to_string(trace).unwrap());
    assert!(!temporary.as_os_str().is_empty());
    assert!(!temporary.exists());
    assert_eq!(fs::read(r.path.join(".git/index")).unwrap(), before);
}

fn traced_index(r: &TestRepo) -> (Git, PathBuf) {
    // Observe the real temp-index path without changing process-global TMPDIR or mocking git.
    let shim = r.path.join(".git/observe-index.sh");
    let trace = r.path.join(".git/observed-index");
    fs::write(&shim, b"#!/bin/sh\nset -eu\nprintf '%s' \"$GIT_INDEX_FILE\" > \"$MORTAR_TEST_INDEX_TRACE\"\nexec git \"$@\"\n").unwrap();
    fs::set_permissions(&shim, fs::Permissions::from_mode(0o700)).unwrap();
    let mut git = r.git.clone();
    git.program = shim;
    git.env.push(("MORTAR_TEST_INDEX_TRACE".into(), trace.as_os_str().to_owned()));
    (git, trace)
}
