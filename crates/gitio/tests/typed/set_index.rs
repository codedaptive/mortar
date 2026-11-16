//! WO-orc-gitio-it-4, S-33 and S-125: Git::set_index sets a folder's index to a tree without touching files or HEAD, keeping sparse folders' skip-worktree bits.
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
fn set_index_save_in_a_sparse_folder_leaves_status_clean_and_the_folder_removes_without_force() {
    // Joins sparse_set -> HEAD moved to a save -> set_index(save tree) -> git status -> worktree_remove(no force).
    let (r, head) = checked_out(&[("root.txt", b"root\n"), ("keep/file", b"base\n"), ("outside/file", b"hidden\n")]);
    let caller_index = fs::read(index_path(&r.git)).unwrap();
    let (folder, linked) = sparse_folder(&r, &head);
    let saved_tree = tree(&r.git, &[("root.txt", b"root\n"), ("keep/file", b"saved work\n"), ("outside/file", b"hidden\n")]);
    let saved = commit(&r.git, &saved_tree, &[head], "save\n");
    set_ref(&linked, "HEAD", &saved);
    fs::write(folder.join("keep/file"), b"saved work\n").unwrap();
    r.git.set_index(&folder, &saved_tree).unwrap();
    assert_eq!(flags(&linked), b"H keep/file\0S outside/file\0H root.txt\0", "out-of-cone paths keep S");
    assert_eq!(linked.run(&["status", "--porcelain"], &RunOpts::default()).unwrap().stdout, b"", "a save is clean");
    assert_eq!(fs::read(folder.join("keep/file")).unwrap(), b"saved work\n");
    assert_eq!(fs::read(folder.join("root.txt")).unwrap(), b"root\n");
    assert!(!folder.join("outside").exists());
    assert_eq!(linked.rev_parse("HEAD").unwrap(), Some(saved));
    assert_eq!(fs::read(index_path(&r.git)).unwrap(), caller_index);
    r.git.worktree_remove(&folder, false).unwrap();
    assert!(!folder.exists());
}

#[test]
fn set_index_sparse_tree_adding_paths_marks_out_of_cone_skip_worktree_keeps_files_and_repeat_is_a_no_op() {
    // Joins sparse_set -> set_index(tree adding in- and out-of-cone paths) -> ls-files -t, write-tree, files, HEAD.
    let (r, head) = checked_out(&[("root.txt", b"root\n"), ("keep/file", b"base\n"), ("outside/file", b"hidden\n")]);
    let caller_index = fs::read(index_path(&r.git)).unwrap();
    let (folder, linked) = sparse_folder(&r, &head);
    fs::write(folder.join("keep/file"), b"dirty sparse work\n").unwrap();
    fs::write(folder.join("keep/untracked"), b"local only\n").unwrap();
    let wanted = tree(&r.git, &[
        ("root.txt", b"new root index\n"), ("keep/file", b"new index\n"), ("keep/new", b"new in-cone\n"),
        ("outside/file", b"new hidden index\n"), ("outside/new", b"new out-of-cone\n"), ("fresh/file", b"new dir\n"),
    ]);
    r.git.set_index(&folder, &wanted).unwrap();
    let expected: &[u8] = b"S fresh/file\0H keep/file\0H keep/new\0S outside/file\0S outside/new\0H root.txt\0";
    assert_eq!(flags(&linked), expected, "every out-of-cone path, old or new, is S");
    assert_eq!(index_tree(&linked), wanted);
    assert_eq!(fs::read(folder.join("keep/file")).unwrap(), b"dirty sparse work\n");
    assert_eq!(fs::read(folder.join("keep/untracked")).unwrap(), b"local only\n");
    assert_eq!(fs::read(folder.join("root.txt")).unwrap(), b"root\n");
    assert!(!folder.join("keep/new").exists());
    assert!(!folder.join("outside").exists());
    assert!(!folder.join("fresh").exists());
    assert_eq!(linked.rev_parse("HEAD").unwrap(), Some(head));
    assert_eq!(fs::read(index_path(&r.git)).unwrap(), caller_index);
    let linked_index = fs::read(index_path(&linked)).unwrap();
    r.git.set_index(&folder, &wanted).unwrap();
    assert_eq!(fs::read(index_path(&linked)).unwrap(), linked_index, "same-tree repeat must not change the index");
    assert_eq!(flags(&linked), expected);
    assert_eq!(fs::read(folder.join("keep/file")).unwrap(), b"dirty sparse work\n");
}

#[test]
fn set_index_leaves_an_out_of_cone_file_present_on_disk_tracked_and_unchanged() {
    // Joins sparse_set -> a file written outside the cone -> set_index -> ls-files -t and the file's bytes.
    let (r, head) = checked_out(&[
        ("root.txt", b"root\n"), ("keep/file", b"base\n"), ("outside/file", b"hidden\n"), ("outside/other", b"other\n"),
    ]);
    let (folder, linked) = sparse_folder(&r, &head);
    fs::create_dir(folder.join("outside")).unwrap();
    fs::write(folder.join("outside/file"), b"present out of cone\n").unwrap();
    let wanted = tree(&r.git, &[
        ("root.txt", b"root\n"), ("keep/file", b"new index\n"), ("outside/file", b"hidden\n"), ("outside/other", b"other\n"),
    ]);
    r.git.set_index(&folder, &wanted).unwrap();
    assert_eq!(flags(&linked), b"H keep/file\0H outside/file\0S outside/other\0H root.txt\0");
    assert_eq!(fs::read(folder.join("outside/file")).unwrap(), b"present out of cone\n");
    assert!(!folder.join("outside/other").exists());
    assert_eq!(fs::read(folder.join("keep/file")).unwrap(), b"base\n");
    assert_eq!(fs::read(folder.join("root.txt")).unwrap(), b"root\n");
    assert_eq!(linked.rev_parse("HEAD").unwrap(), Some(head));
}

#[test]
fn set_index_keeps_an_out_of_cone_file_on_disk_whose_bytes_equal_the_trees_blob() {
    // Joins sparse_set -> an out-of-cone file matching the tree, freshly stat'ed -> set_index -> ls-files -t and bytes.
    let (r, head) = checked_out(&[
        ("root.txt", b"root\n"), ("keep/file", b"base\n"), ("outside/file", b"hidden\n"), ("outside/other", b"other\n"),
    ]);
    let (folder, linked) = sparse_folder(&r, &head);
    fs::create_dir(folder.join("outside")).unwrap();
    fs::write(folder.join("outside/file"), b"hidden\n").unwrap();
    // Git clears the S bit of a tracked file present on disk, and status then writes its fresh stat to the index, so
    // only set_index's own steps decide whether it survives.
    linked.run(&["status", "--porcelain"], &RunOpts::default()).unwrap();
    let wanted = tree(&r.git, &[
        ("root.txt", b"root\n"), ("keep/file", b"new index\n"), ("outside/file", b"hidden\n"), ("outside/other", b"other\n"),
    ]);
    r.git.set_index(&folder, &wanted).unwrap();
    assert_eq!(fs::read(folder.join("outside/file")).unwrap(), b"hidden\n", "a working file is never deleted");
    assert_eq!(flags(&linked), b"H keep/file\0H outside/file\0S outside/other\0H root.txt\0");
    assert!(!folder.join("outside/other").exists());
    assert_eq!(fs::read(folder.join("keep/file")).unwrap(), b"base\n");
    assert_eq!(linked.rev_parse("HEAD").unwrap(), Some(head));
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

/// A linked worktree on `head` made cone-sparse on `keep`, checked to really be sparse before the test acts.
fn sparse_folder(r: &GitRepo, head: &Oid) -> (PathBuf, Git) {
    let folder = r.path.join("sparse folder");
    r.git.worktree_add(&folder, head, None).unwrap();
    r.git.sparse_set(&folder, &["keep".into()]).unwrap();
    let linked = Git { repo: folder.clone(), ..r.git.clone() };
    assert!(!folder.join("outside").exists());
    assert!(flags(&linked).starts_with(b"H keep/file\0S outside/file\0"), "the fixture must really be sparse");
    (folder, linked)
}

fn flags(git: &Git) -> Vec<u8> {
    git.run(&["ls-files", "-t", "-z"], &RunOpts::default()).unwrap().stdout
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
