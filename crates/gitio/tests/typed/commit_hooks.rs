use crate::fixture::{commit, repo, set_ref, tree};
use crate::support::GitRepo;
use gitio::{CommitHooksOutcome, Git, HookRefusal, Oid, RunOpts};
use std::{collections::BTreeMap, fs, path::{Path, PathBuf}};
use testkit::write_git_shim;

fn hook(r: &GitRepo, name: &str, script: &[u8]) {
    write_git_shim(&r.path.join(".git/hooks").join(name), script).unwrap();
}

fn refused(hook: &'static str, stdout: &[u8], stderr: &[u8]) -> CommitHooksOutcome {
    CommitHooksOutcome::Refused(HookRefusal {
        hook, code: Some(1), stdout: stdout.to_vec(), stderr: stderr.to_vec(),
    })
}

fn working_files(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(root: &Path, dir: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path == root.join(".git") { continue; }
            if path.is_dir() {
                visit(root, &path, files);
            } else {
                files.insert(path.strip_prefix(root).unwrap().to_path_buf(), fs::read(&path).unwrap());
            }
        }
    }
    let mut files = BTreeMap::new();
    visit(root, root, &mut files);
    files
}

fn commit_hooks_in_a_cone_sparse_folder_preserves_worktree_and_shows_staged_addition(sparse_index: bool) {
    let r = repo();
    let base_files: &[(&str, &[u8])] = &[
        ("keep/file", b"keep\n"), ("outside/file", b"outside\n"), ("outside/clean present", b"clean\n"),
        ("outside/dirty", b"dirty\n"), ("hidden/file", b"hidden\n"),
    ];
    let base_tree = tree(&r.git, base_files);
    let base = commit(&r.git, &base_tree, &[], "base\n");
    set_ref(&r.git, "refs/heads/main", &base);
    r.git.run(&["checkout", "--force", "main"], &RunOpts::default()).unwrap();
    if sparse_index {
        r.git.run(&["config", "index.sparse", "true"], &RunOpts::default()).unwrap();
    }
    r.git.sparse_set(&r.path, &["keep".into()]).unwrap();
    fs::create_dir(r.path.join("outside")).unwrap();
    fs::write(r.path.join("outside/clean present"), b"clean\n").unwrap();
    assert_eq!(r.git.run(&["status", "--porcelain"], &RunOpts::default()).unwrap().stdout, b"");
    fs::write(r.path.join("outside/dirty"), b"edited\n").unwrap();
    assert_eq!(r.git.run(&["status", "--porcelain"], &RunOpts::default()).unwrap().stdout, b" M outside/dirty\n");
    if sparse_index {
        let paths = r.git.run(&["ls-files", "--sparse", "-z"], &RunOpts::default()).unwrap().stdout;
        assert!(paths.split(|b| *b == 0).any(|path| path == b"hidden/"), "index is sparse: {paths:?}");
    }
    let contents = tree(&r.git, &[
        ("keep/file", b"keep\n"), ("outside/file", b"outside\n"), ("hidden/file", b"hidden\n"),
        ("outside/clean present", b"clean\n"), ("outside/dirty", b"dirty\n"), ("outside/new", b"new\n"),
    ]);
    hook(&r, "pre-commit", b"#!/bin/sh\ngit status --porcelain > .git/hook-status || exit $?\ncat .git/hook-status\nif grep -Eq '^(D.|.D)' .git/hook-status; then exit 1; fi\n");
    let index = fs::read(r.path.join(".git/index")).unwrap();
    let files = working_files(&r.path);
    assert_eq!(files, BTreeMap::from([
        (PathBuf::from("keep/file"), b"keep\n".to_vec()),
        (PathBuf::from("outside/clean present"), b"clean\n".to_vec()),
        (PathBuf::from("outside/dirty"), b"edited\n".to_vec()),
    ]));
    let outcome = r.git.commit_hooks(&contents, "original\n").unwrap();
    assert_eq!(fs::read(r.path.join(".git/index")).unwrap(), index);
    assert_eq!(working_files(&r.path), files);
    assert_eq!(outcome, CommitHooksOutcome::Message("original\n".into()));
    assert_eq!(fs::read(r.path.join(".git/hook-status")).unwrap(), b" M outside/dirty\nA  outside/new\n");
}

#[test]
fn commit_hooks_in_a_cone_sparse_folder() {
    commit_hooks_in_a_cone_sparse_folder_preserves_worktree_and_shows_staged_addition(false);
}
#[test]
fn commit_hooks_in_a_cone_sparse_folder_with_sparse_index() {
    commit_hooks_in_a_cone_sparse_folder_preserves_worktree_and_shows_staged_addition(true);
}

#[test]
fn pre_commit_refusal_keeps_streams_separate_and_stops_commit_msg() {
    let r = repo();
    let contents = tree(&r.git, &[("file", b"contents\n")]);
    hook(&r, "pre-commit", b"#!/bin/sh\nprintf 'pre out\n'\nprintf 'pre err\n' >&2\nexit 1\n");
    hook(&r, "commit-msg", b"#!/bin/sh\ntouch .git/commit-msg-ran\n");
    assert_eq!(r.git.commit_hooks(&contents, "original\n").unwrap(),
        refused("pre-commit", b"pre out\n", b"pre err\n"));
    assert!(!r.path.join(".git/commit-msg-ran").exists());
}

fn index_is_tree_at_worktree_top(from_subfolder: bool) {
    let r = repo();
    let base_tree = tree(&r.git, &[("base-only", b"base\n")]);
    let base = commit(&r.git, &base_tree, &[], "base\n");
    set_ref(&r.git, "refs/heads/main", &base);
    r.git.run(&["checkout", "--force", "main"], &RunOpts::default()).unwrap();
    let contents = tree(&r.git, &[("alpha", b"a\n"), ("nested/beta", b"b\n")]);
    let original_index = fs::read(r.path.join(".git/index")).unwrap();
    fs::create_dir(r.path.join("sub")).unwrap();
    hook(&r, "pre-commit", b"#!/bin/sh\nset -e\n[ \"$#\" -eq 0 ]\ngit ls-files > .git/hook-files\npwd -P > .git/hook-cwd\nprintf '%s' \"$GIT_INDEX_FILE\" > .git/hook-index\nprintf '%s' \"$GIT_EDITOR\" > .git/hook-editor\ngit config --get hooktest.value > .git/hook-config\n");
    let mut git = Git { repo: if from_subfolder { r.path.join("sub") } else { r.path.clone() }, ..r.git.clone() };
    git.env.extend([
        ("GIT_CONFIG_COUNT".into(), "1".into()),
        ("GIT_CONFIG_KEY_0".into(), "hooktest.value".into()),
        ("GIT_CONFIG_VALUE_0".into(), "from-git-env".into()),
    ]);
    assert_eq!(git.commit_hooks(&contents, "original\n").unwrap(), CommitHooksOutcome::Message("original\n".into()));
    assert_eq!(fs::read(r.path.join(".git/hook-files")).unwrap(), b"alpha\nnested/beta\n");
    assert_eq!(fs::read_to_string(r.path.join(".git/hook-cwd")).unwrap().trim_end(),
        r.path.canonicalize().unwrap().to_str().unwrap());
    assert_eq!(fs::read(r.path.join(".git/hook-editor")).unwrap(), b":");
    assert_eq!(fs::read(r.path.join(".git/hook-config")).unwrap(), b"from-git-env\n");
    assert_eq!(fs::read(r.path.join(".git/index")).unwrap(), original_index);
    let temp_index = PathBuf::from(fs::read_to_string(r.path.join(".git/hook-index")).unwrap());
    assert!(!temp_index.as_os_str().is_empty());
    assert_ne!(temp_index, r.path.join(".git/index"));
    assert!(!temp_index.exists());
}

#[test]
fn index_is_tree_from_root() { index_is_tree_at_worktree_top(false); }
#[test]
fn index_is_tree_from_subfolder() { index_is_tree_at_worktree_top(true); }

#[test]
fn commit_msg_refusal_keeps_streams_separate() {
    let r = repo();
    let contents = tree(&r.git, &[("file", b"contents\n")]);
    hook(&r, "commit-msg", b"#!/bin/sh\nprintf 'message out\n'\nprintf 'message err\n' >&2\nexit 1\n");
    assert_eq!(r.git.commit_hooks(&contents, "original\n").unwrap(),
        refused("commit-msg", b"message out\n", b"message err\n"));
}

#[test]
fn commit_msg_receives_original_and_returns_rewritten_text() {
    let r = repo();
    let contents = tree(&r.git, &[("file", b"contents\n")]);
    hook(&r, "commit-msg", b"#!/bin/sh\n[ \"$#\" -eq 1 ] || exit 1\ncat \"$1\" > .git/original-message\nprintf 'rewritten message\nsecond line\n' > \"$1\"\n");
    assert_eq!(r.git.commit_hooks(&contents, "original message\n").unwrap(),
        CommitHooksOutcome::Message("rewritten message\nsecond line\n".into()));
    assert_eq!(fs::read(r.path.join(".git/original-message")).unwrap(), b"original message\n");
}

fn relative_hooks_path_overrides_git_dir_hooks(from_subfolder: bool) {
    let r = repo();
    let contents = tree(&r.git, &[("file", b"contents\n")]);
    r.git.run(&["config", "core.hooksPath", "my-hooks"], &RunOpts::default()).unwrap();
    fs::create_dir(r.path.join("my-hooks")).unwrap();
    fs::create_dir(r.path.join("sub")).unwrap();
    write_git_shim(&r.path.join("my-hooks/pre-commit"), b"#!/bin/sh\nprintf 'custom out\n'\nexit 1\n").unwrap();
    hook(&r, "pre-commit", b"#!/bin/sh\ntouch .git/default-hook-ran\nexit 1\n");
    let git = Git { repo: if from_subfolder { r.path.join("sub") } else { r.path.clone() }, ..r.git.clone() };
    assert_eq!(git.commit_hooks(&contents, "original\n").unwrap(),
        refused("pre-commit", b"custom out\n", b""));
    assert!(!r.path.join(".git/default-hook-ran").exists());
}

#[test]
fn relative_hooks_path_from_root() { relative_hooks_path_overrides_git_dir_hooks(false); }
#[test]
fn relative_hooks_path_from_subfolder() { relative_hooks_path_overrides_git_dir_hooks(true); }

#[test]
fn bare_repository_hooks_run_in_the_git_dir() {
    let r = repo();
    let bare = r.path.join("bare.git");
    r.git.run(&["init", "--quiet", "--bare", bare.to_str().unwrap()], &RunOpts::default()).unwrap();
    let git = Git { repo: bare.clone(), ..r.git.clone() };
    git.run(&["config", "core.sparseCheckout", "true"], &RunOpts::default()).unwrap();
    let output = git.run(&["mktree"], &RunOpts { stdin: Some(Vec::new()), ..RunOpts::default() }).unwrap();
    let contents = Oid::parse(String::from_utf8(output.stdout).unwrap().trim_end()).unwrap();
    write_git_shim(&bare.join("hooks/pre-commit"), b"#!/bin/sh\npwd -P > hook-cwd\nprintf 'bare out\n'\nexit 1\n").unwrap();
    assert_eq!(git.commit_hooks(&contents, "original\n").map_err(|err| format!("{err:?}")),
        Ok(refused("pre-commit", b"bare out\n", b"")));
    assert_eq!(fs::read_to_string(bare.join("hook-cwd")).unwrap().trim_end(),
        bare.canonicalize().unwrap().to_str().unwrap());
}

fn absent_or_non_executable_hooks_pass(non_executable: bool) {
    let r = repo();
    let contents = tree(&r.git, &[("file", b"contents\n")]);
    if non_executable {
        for name in ["pre-commit", "commit-msg"] {
            fs::write(r.path.join(".git/hooks").join(name), b"#!/bin/sh\nexit 1\n").unwrap();
        }
    }
    assert_eq!(r.git.commit_hooks(&contents, "unchanged message\n").unwrap(),
        CommitHooksOutcome::Message("unchanged message\n".into()));
}

#[test]
fn missing_hooks_pass() { absent_or_non_executable_hooks_pass(false); }
#[test]
fn non_executable_hooks_pass() { absent_or_non_executable_hooks_pass(true); }

fn temporary_files_are_removed(refusal: Option<&'static str>) {
    let r = repo();
    let contents = tree(&r.git, &[("file", b"contents\n")]);
    let pre_script: &[u8] = if refusal == Some("pre-commit") {
        b"#!/bin/sh\nprintf '%s' \"$GIT_INDEX_FILE\" > .git/pre-index\nexit 1\n"
    } else {
        b"#!/bin/sh\nprintf '%s' \"$GIT_INDEX_FILE\" > .git/pre-index\n"
    };
    let msg_script: &[u8] = if refusal == Some("commit-msg") {
        b"#!/bin/sh\nprintf '%s' \"$GIT_INDEX_FILE\" > .git/msg-index\nprintf '%s' \"$GIT_EDITOR\" > .git/msg-editor\nprintf '%s' \"$1\" > .git/msg-path\nexit 1\n"
    } else {
        b"#!/bin/sh\nprintf '%s' \"$GIT_INDEX_FILE\" > .git/msg-index\nprintf '%s' \"$GIT_EDITOR\" > .git/msg-editor\nprintf '%s' \"$1\" > .git/msg-path\n"
    };
    hook(&r, "pre-commit", pre_script);
    hook(&r, "commit-msg", msg_script);
    let expected = match refusal {
        Some(name) => refused(name, b"", b""),
        None => CommitHooksOutcome::Message("original\n".into()),
    };
    assert_eq!(r.git.commit_hooks(&contents, "original\n").unwrap(), expected);
    let index = fs::read_to_string(r.path.join(".git/pre-index")).unwrap();
    assert!(!index.is_empty());
    assert!(!PathBuf::from(&index).exists());
    if refusal == Some("pre-commit") {
        assert!(!r.path.join(".git/msg-path").exists());
    } else {
        assert_eq!(fs::read_to_string(r.path.join(".git/msg-index")).unwrap(), index);
        assert_eq!(fs::read(r.path.join(".git/msg-editor")).unwrap(), b":");
        let message = fs::read_to_string(r.path.join(".git/msg-path")).unwrap();
        assert!(!message.is_empty());
        assert!(!PathBuf::from(message).exists());
    }
}

#[test]
fn temporary_files_removed_on_success() { temporary_files_are_removed(None); }
#[test]
fn temporary_files_removed_on_pre_commit_refusal() { temporary_files_are_removed(Some("pre-commit")); }
#[test]
fn temporary_files_removed_on_commit_msg_refusal() { temporary_files_are_removed(Some("commit-msg")); }
