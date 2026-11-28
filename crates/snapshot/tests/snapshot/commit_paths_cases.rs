use crate::support::{blob, hook_script, run, tree_names, write, Fixture};
use gitio::HookRefusal;
use oplog::oplog_path;
use snapshot::{commit_paths, commit_paths_with_hooks, CommitPathsError, CommitPathsHooksError, CommitPathsOutcome};
use std::fs;
use std::os::unix::fs::symlink;
use std::path::PathBuf;

fn refs(f: &Fixture) -> Vec<u8> {
    run(&f.git, &["for-each-ref", "--format=%(refname) %(objectname)"])
}

fn objects(f: &Fixture) -> Vec<u8> {
    run(&f.git, &["cat-file", "--batch-all-objects", "--batch-check=%(objecttype) %(objectname)"])
}

fn commits(f: &Fixture) -> Vec<String> {
    std::str::from_utf8(&objects(f)).unwrap().lines()
        .filter(|line| line.starts_with("commit "))
        .map(str::to_owned).collect()
}

fn log(f: &Fixture) -> Option<Vec<u8>> {
    match fs::read(oplog_path(&f.common)) {
        Ok(bytes) => Some(bytes),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => panic!("read oplog: {error}"),
    }
}

#[test]
fn commit_paths_added_changed_deleted_and_explicit_excluded_files() {
    let f = Fixture::new(&[("changed.txt", b"old\n"), ("deleted.txt", b"gone\n")]);
    write(&f.repo.path, "added.txt", b"added\n");
    write(&f.repo.path, "changed.txt", b"changed\n");
    fs::remove_file(f.repo.path.join("deleted.txt")).unwrap();
    write(&f.repo.path, "target/named.txt", b"included build output\n");
    let large = vec![b'x'; 5 * 1024 * 1024 + 1];
    write(&f.repo.path, "large.bin", &large);
    let paths: Vec<PathBuf> = ["added.txt", "changed.txt", "deleted.txt", "target/named.txt", "large.bin"]
        .into_iter().map(PathBuf::from).collect();
    let (commit, tree) = match commit_paths(&f.git, &f.repo.path, &f.head, &paths, "Named paths\n").unwrap() {
        CommitPathsOutcome::Committed { commit, tree } => (commit, tree),
        CommitPathsOutcome::Same => panic!("changed paths must produce a commit"),
    };
    let info = f.git.commit_info(&commit).unwrap();
    assert_eq!(info.parents, vec![f.head.clone()]);
    assert_eq!(info.message, "Named paths\n");
    assert_eq!(info.tree, tree);
    assert_eq!(tree_names(&f.git, &tree), ["added.txt", "changed.txt", "large.bin", "root.txt", "target/named.txt"]);
    assert_eq!(blob(&f.git, &tree, "added.txt"), b"added\n");
    assert_eq!(blob(&f.git, &tree, "changed.txt"), b"changed\n");
    assert_eq!(blob(&f.git, &tree, "target/named.txt"), b"included build output\n");
    assert_eq!(blob(&f.git, &tree, "large.bin"), large);
    assert!(run(&f.git, &["diff", "--name-only", "--", "added.txt", "changed.txt", "deleted.txt", "target/named.txt", "large.bin"]).is_empty());
    assert_eq!(run(&f.git, &["status", "--porcelain=v1", "-z", "--", "added.txt", "changed.txt", "deleted.txt", "target/named.txt", "large.bin"]),
        b"A  added.txt\0M  changed.txt\0D  deleted.txt\0A  large.bin\0A  target/named.txt\0");
}

#[test]
fn commit_paths_preserves_unnamed_worktree_and_index_edits() {
    let f = Fixture::new(&[("named.txt", b"old named\n"), ("unstaged.txt", b"parent unstaged\n"), ("staged.txt", b"parent staged\n")]);
    write(&f.repo.path, "unstaged.txt", b"worktree only\n");
    write(&f.repo.path, "staged.txt", b"staged content\n");
    run(&f.git, &["add", "staged.txt"]);
    write(&f.repo.path, "named.txt", b"new named\n");
    let tree = match commit_paths(&f.git, &f.repo.path, &f.head, &["named.txt".into()], "Only named\n").unwrap() {
        CommitPathsOutcome::Committed { tree, .. } => tree,
        CommitPathsOutcome::Same => panic!("named change must produce a commit"),
    };
    assert_eq!(blob(&f.git, &tree, "named.txt"), b"new named\n");
    assert_eq!(blob(&f.git, &tree, "unstaged.txt"), b"parent unstaged\n");
    assert_eq!(blob(&f.git, &tree, "staged.txt"), b"parent staged\n");
    assert_eq!(fs::read(f.repo.path.join("unstaged.txt")).unwrap(), b"worktree only\n");
    assert_eq!(fs::read(f.repo.path.join("staged.txt")).unwrap(), b"staged content\n");
    assert_eq!(run(&f.git, &["show", ":unstaged.txt"]), b"parent unstaged\n");
    assert_eq!(run(&f.git, &["show", ":staged.txt"]), b"staged content\n");
    assert_eq!(run(&f.git, &["status", "--porcelain=v1", "-z", "--", "staged.txt", "unstaged.txt"]), b"M  staged.txt\0 M unstaged.txt\0");
}

#[test]
fn commit_paths_same_leaves_refs_head_index_and_objects_unchanged() {
    let f = Fixture::new(&[]);
    write(&f.repo.path, "root.txt", b"staged but not in worktree\n");
    run(&f.git, &["add", "root.txt"]);
    write(&f.repo.path, "root.txt", b"base\n");
    let before_refs = refs(&f);
    let before_head = run(&f.git, &["rev-parse", "HEAD"]);
    let before_index = fs::read(f.common.join("index")).unwrap();
    let before_objects = objects(&f);
    let before_log = log(&f);
    assert!(matches!(commit_paths(&f.git, &f.repo.path, &f.head, &["root.txt".into()], "No change\n").unwrap(), CommitPathsOutcome::Same));
    assert_eq!(refs(&f), before_refs);
    assert_eq!(run(&f.git, &["rev-parse", "HEAD"]), before_head);
    assert_eq!(fs::read(f.common.join("index")).unwrap(), before_index);
    assert_eq!(objects(&f), before_objects);
    assert_eq!(log(&f), before_log);
}

macro_rules! outside_case {
    ($name:ident, $path:expr, $pattern:pat, $check:expr) => {
        #[test]
        fn $name() {
            let f = Fixture::new(&[]);
            fs::create_dir(f.repo.path.join("inside")).unwrap();
            symlink(f.repo.path.join("inside"), f.repo.path.join("link")).unwrap();
            let path: PathBuf = $path(&f);
            let before_refs = refs(&f);
            let before_head = run(&f.git, &["rev-parse", "HEAD"]);
            let before_index = fs::read(f.common.join("index")).unwrap();
            let before_log = log(&f);
            let before_objects = objects(&f);
            match commit_paths(&f.git, &f.repo.path, &f.head, std::slice::from_ref(&path), "Refused\n") {
                Err($pattern) => $check(&path),
                Err(other) => panic!("wrong refusal: {other:?}"),
                Ok(_) => panic!("outside path was accepted"),
            }
            assert_eq!(refs(&f), before_refs);
            assert_eq!(run(&f.git, &["rev-parse", "HEAD"]), before_head);
            assert_eq!(fs::read(f.common.join("index")).unwrap(), before_index);
            assert_eq!(log(&f), before_log);
            assert_eq!(objects(&f), before_objects);
        }
    };
}

outside_case!(commit_paths_refuses_absolute, |f: &Fixture| f.repo.path.join("x"),
    CommitPathsError::Absolute { path: actual }, |path: &PathBuf| assert_eq!(&actual, path));
outside_case!(commit_paths_refuses_parent_part, |_f: &Fixture| PathBuf::from("../x"),
    CommitPathsError::ParentPart { path: actual }, |path: &PathBuf| assert_eq!(&actual, path));
outside_case!(commit_paths_refuses_symlink_ancestor, |_f: &Fixture| PathBuf::from("link/x"),
    CommitPathsError::ThroughSymlink { path: actual, link }, |path: &PathBuf| {
        assert_eq!(&actual, path);
        assert_eq!(link, PathBuf::from("link"));
    });

#[test]
fn commit_paths_committed_moves_no_refs_and_writes_no_oplog_entry() {
    let f = Fixture::new(&[]);
    run(&f.git, &["update-ref", "refs/mortar/test/existing", f.head.as_str()]);
    write(&f.repo.path, "root.txt", b"new content\n");
    let before_refs = refs(&f);
    let before_head = run(&f.git, &["rev-parse", "HEAD"]);
    let before_log = log(&f);
    assert!(matches!(commit_paths(&f.git, &f.repo.path, &f.head, &["root.txt".into()], "Objects only\n").unwrap(), CommitPathsOutcome::Committed { .. }));
    assert_eq!(refs(&f), before_refs);
    assert_eq!(run(&f.git, &["rev-parse", "HEAD"]), before_head);
    assert_eq!(log(&f), before_log);
}

#[test]
fn commit_paths_refuses_ignored_named_file_without_a_commit() {
    let f = Fixture::new(&[(".gitignore", b"ignored.txt\n")]);
    write(&f.repo.path, "ignored.txt", b"ignored content\n");
    let before = commits(&f);
    match commit_paths(&f.git, &f.repo.path, &f.head, &["ignored.txt".into()], "Ignored\n") {
        Err(CommitPathsError::NotAdded { path }) => assert_eq!(path, PathBuf::from("ignored.txt")),
        Err(other) => panic!("wrong refusal: {other:?}"),
        Ok(_) => panic!("ignored path was accepted"),
    }
    assert_eq!(commits(&f), before);
}

#[test]
fn hooks_refusing_pre_commit_stops_commit_paths_with_its_output_and_leaves_index_and_commits() {
    // Case 2.
    let f = Fixture::new(&[]);
    write(&f.repo.path, "root.txt", b"refused path\n");
    hook_script(&f, "pre-commit", b"printf 'out-text\\n'\nprintf 'err-text\\n' >&2\nexit 3\n");
    let before_index = fs::read(f.common.join("index")).unwrap();
    let before_commits = commits(&f);
    let before_refs = refs(&f);
    match commit_paths_with_hooks(&f.git, &f.repo.path, &f.head, &["root.txt".into()], "Refused\n") {
        Err(CommitPathsHooksError::HookRefused(refusal)) => assert_eq!(refusal,
            HookRefusal { hook: "pre-commit", code: Some(3), stdout: b"out-text\n".to_vec(), stderr: b"err-text\n".to_vec() }),
        other => panic!("expected HookRefused, got {other:?}"),
    }
    assert_eq!(commits(&f), before_commits);
    assert_eq!(fs::read(f.common.join("index")).unwrap(), before_index);
    assert_eq!(refs(&f), before_refs);
}

#[test]
fn hooks_refusing_commit_msg_stops_commit_paths_without_a_commit() {
    // Case 3 (commit_paths).
    let f = Fixture::new(&[]);
    write(&f.repo.path, "root.txt", b"refused message\n");
    hook_script(&f, "commit-msg", b"exit 1\n");
    let before_index = fs::read(f.common.join("index")).unwrap();
    let before_commits = commits(&f);
    match commit_paths_with_hooks(&f.git, &f.repo.path, &f.head, &["root.txt".into()], "Refused message\n") {
        Err(CommitPathsHooksError::HookRefused(refusal)) => assert_eq!((refusal.hook, refusal.code), ("commit-msg", Some(1))),
        other => panic!("expected HookRefused, got {other:?}"),
    }
    assert_eq!(commits(&f), before_commits);
    assert_eq!(fs::read(f.common.join("index")).unwrap(), before_index);
}

#[test]
fn hooks_commit_msg_rewrite_is_commit_paths_commit_message() {
    // Case 4 (commit_paths).
    let f = Fixture::new(&[]);
    write(&f.repo.path, "root.txt", b"rewritten path\n");
    hook_script(&f, "commit-msg", b"printf 'rewritten by hook\\n' > \"$1\"\n");
    let commit = match commit_paths_with_hooks(&f.git, &f.repo.path, &f.head, &["root.txt".into()], "Built message\n").unwrap() {
        CommitPathsOutcome::Committed { commit, .. } => commit,
        CommitPathsOutcome::Same => panic!("a changed path must produce a commit"),
    };
    assert_eq!(f.git.commit_info(&commit).unwrap().message, "rewritten by hook\n");
}
