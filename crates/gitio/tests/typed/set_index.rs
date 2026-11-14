//! WO-orc-gitio-it-4, S-33: Git::set_index joins real read-tree/index semantics without checking files out.
use crate::fixture::{commit, repo, set_ref, tree};
use crate::support::GitRepo;
use gitio::{Git, Oid, RunOpts};
use std::{fs, path::PathBuf};

#[test]
fn set_index_replaces_only_the_named_worktrees_index_and_repeat_preserves_dirty_files_and_head() {
    // Joins worktree_add -> set_index(folder, tree) -> exact write-tree plus file/HEAD/index byte observations.
    let (r, base) = checked_out(&[("tracked", b"base\n"), ("removed", b"keep on disk\n")]);
    let main_index = fs::read(index_path(&r.git)).unwrap();
    let folder = r.path.join("linked folder");
    r.git.worktree_add(&folder, &base, None).unwrap();
    let linked = Git { repo: folder.clone(), ..r.git.clone() };
    linked.run(&["checkout", "--force", "HEAD"], &RunOpts::default()).unwrap();
    fs::write(folder.join("tracked"), b"dirty work must survive\n").unwrap();
    fs::write(folder.join("untracked"), b"local only\n").unwrap();
    let wanted = tree(&r.git, &[("tracked", b"index version\n"), ("added", b"new index-only file\n")]);
    assert_ne!(index_tree(&linked), wanted);
    r.git.set_index(&folder, &wanted).unwrap();
    assert_eq!(index_tree(&linked), wanted, "added/changed/deleted index entries must match the entire tree");
    assert_eq!(linked.run(&["ls-files", "-z"], &RunOpts::default()).unwrap().stdout, b"added\0tracked\0");
    assert_eq!(fs::read(folder.join("tracked")).unwrap(), b"dirty work must survive\n");
    assert_eq!(fs::read(folder.join("removed")).unwrap(), b"keep on disk\n");
    assert_eq!(fs::read(folder.join("untracked")).unwrap(), b"local only\n");
    assert!(!folder.join("added").exists());
    assert_eq!(linked.rev_parse("HEAD").unwrap(), Some(base.clone()));
    assert_eq!(r.git.rev_parse("HEAD").unwrap(), Some(base.clone()));
    assert_eq!(fs::read(index_path(&r.git)).unwrap(), main_index, "Git.repo's index is not folder's index");
    let linked_index = fs::read(index_path(&linked)).unwrap();
    r.git.set_index(&folder, &wanted).unwrap();
    assert_eq!(fs::read(index_path(&linked)).unwrap(), linked_index, "same-tree repeat must not change the index");
    assert_eq!(fs::read(folder.join("tracked")).unwrap(), b"dirty work must survive\n");
    assert_eq!(linked.rev_parse("HEAD").unwrap(), Some(base));
    assert_eq!(fs::read(index_path(&r.git)).unwrap(), main_index);
}

#[test]
fn set_index_unknown_and_blob_objects_return_git_error_without_changing_index_files_or_head() {
    // Joins set_index -> real git failure -> typed GitError code/args/stderr and unchanged work.
    let (r, head) = checked_out(&[("file", b"base\n")]);
    fs::write(r.path.join("file"), b"dirty preserved\n").unwrap();
    let written = r.git.run(&["hash-object", "-w", "--stdin"], &RunOpts {
        stdin: Some(b"this is a blob, not a tree\n".to_vec()), ..RunOpts::default()
    }).unwrap();
    let blob = parse_oid(&written.stdout);
    let before = fs::read(index_path(&r.git)).unwrap();
    for invalid in [Oid::parse(&"0".repeat(40)).unwrap(), blob] {
        let args = ["read-tree", "--end-of-options", invalid.as_str()];
        // Raw git is the exit/stderr oracle, with an isolated index so the oracle cannot repair the real one.
        let oracle = r.git.run(&args, &RunOpts {
            answer_codes: vec![1, 128],
            env: vec![("GIT_INDEX_FILE".into(), r.path.join(".git/error-oracle-index").into_os_string())],
            ..RunOpts::default()
        }).unwrap();
        assert_ne!(oracle.status, 0, "the independently created object must not be a usable tree");
        let error = r.git.set_index(&r.path, &invalid).unwrap_err();
        assert_eq!(error.args, args);
        assert_eq!(error.code, Some(oracle.status));
        assert_eq!(error.stderr.as_bytes(), oracle.stderr);
        assert!(error.stderr.contains("fatal:") && error.stderr.contains(invalid.as_str()), "{error:?}");
        assert_eq!(fs::read(index_path(&r.git)).unwrap(), before);
        assert_eq!(fs::read(r.path.join("file")).unwrap(), b"dirty preserved\n");
        assert_eq!(r.git.rev_parse("HEAD").unwrap(), Some(head.clone()));
    }
}

#[test]
fn set_index_sparse_flags_match_independent_read_tree_and_neither_worktree_is_filled_or_overwritten() {
    // Joins sparse_set -> set_index -> ls-files skip-worktree flags against a separate real-git oracle worktree.
    let (r, head) = checked_out(&[("root.txt", b"root\n"), ("keep/file", b"base\n"), ("outside/file", b"hidden\n")]);
    let caller_index = fs::read(index_path(&r.git)).unwrap();
    let actual_path = r.path.join("actual sparse");
    let oracle_path = r.path.join("oracle sparse");
    for folder in [&actual_path, &oracle_path] {
        r.git.worktree_add(folder, &head, None).unwrap();
        r.git.sparse_set(folder, &["keep".into()]).unwrap();
        assert!(!folder.join("outside").exists());
        fs::write(folder.join("keep/file"), b"dirty sparse work\n").unwrap();
    }
    let actual = Git { repo: actual_path.clone(), ..r.git.clone() };
    let oracle = Git { repo: oracle_path.clone(), ..r.git.clone() };
    let flags_before = actual.run(&["ls-files", "-t", "-z"], &RunOpts::default()).unwrap().stdout;
    assert_eq!(flags_before, b"H keep/file\0S outside/file\0H root.txt\0", "the fixture must really be sparse");
    let wanted = tree(&r.git, &[
        ("root.txt", b"new root index\n"), ("keep/file", b"new index\n"), ("keep/new", b"new in-cone\n"),
        ("outside/file", b"new hidden index\n"), ("outside/new", b"new out-of-cone\n"),
    ]);
    oracle.run(&["read-tree", "--end-of-options", wanted.as_str()], &RunOpts::default()).unwrap();
    r.git.set_index(&actual_path, &wanted).unwrap();
    let expected_flags = oracle.run(&["ls-files", "-t", "-z"], &RunOpts::default()).unwrap().stdout;
    assert_eq!(actual.run(&["ls-files", "-t", "-z"], &RunOpts::default()).unwrap().stdout, expected_flags);
    for git in [&actual, &oracle] {
        assert_eq!(index_tree(git), wanted);
        assert_eq!(git.rev_parse("HEAD").unwrap(), Some(head.clone()));
        assert_eq!(fs::read(git.repo.join("keep/file")).unwrap(), b"dirty sparse work\n");
        assert_eq!(fs::read(git.repo.join("root.txt")).unwrap(), b"root\n");
        assert!(!git.repo.join("keep/new").exists());
        assert!(!git.repo.join("outside").exists());
    }
    assert_eq!(fs::read(index_path(&r.git)).unwrap(), caller_index);
}

#[test]
fn set_index_honors_a_callers_explicit_index_without_changing_the_folders_default_index() {
    // Joins caller Git.env -> set_index -> alternate real index, preserving default index, files and HEAD.
    let (r, head) = checked_out(&[("file", b"base\n")]);
    fs::write(r.path.join("file"), b"dirty\n").unwrap();
    let before = fs::read(index_path(&r.git)).unwrap();
    let wanted = tree(&r.git, &[("alternate", b"index-only\n")]);
    let alternate = r.path.join(".git/explicit-index");
    assert!(!alternate.exists());
    let mut caller = r.git.clone();
    caller.env.push(("GIT_INDEX_FILE".into(), alternate.clone().into_os_string()));
    caller.set_index(&r.path, &wanted).unwrap();
    assert!(alternate.is_file());
    assert_eq!(index_tree(&caller), wanted);
    assert_eq!(fs::read(index_path(&r.git)).unwrap(), before);
    assert_eq!(fs::read(r.path.join("file")).unwrap(), b"dirty\n");
    assert!(!r.path.join("alternate").exists());
    assert_eq!(r.git.rev_parse("HEAD").unwrap(), Some(head));
}

fn checked_out(files: &[(&str, &[u8])]) -> (GitRepo, Oid) {
    let r = repo();
    let contents = tree(&r.git, files);
    let head = commit(&r.git, &contents, &[], "set_index fixture\n");
    set_ref(&r.git, "refs/heads/main", &head);
    r.git.run(&["checkout", "--force", "main"], &RunOpts::default()).unwrap();
    (r, head)
}

fn index_tree(git: &Git) -> Oid {
    parse_oid(&git.run(&["write-tree"], &RunOpts::default()).unwrap().stdout)
}

fn parse_oid(bytes: &[u8]) -> Oid {
    Oid::parse(std::str::from_utf8(bytes).unwrap().trim_end()).unwrap()
}

fn index_path(git: &Git) -> PathBuf {
    let bytes = git.run(&["rev-parse", "--path-format=absolute", "--git-path", "index"], &RunOpts::default()).unwrap().stdout;
    PathBuf::from(std::str::from_utf8(&bytes).unwrap().trim_end())
}
