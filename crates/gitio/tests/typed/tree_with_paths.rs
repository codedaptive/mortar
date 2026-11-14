//! WO-orc-gitio-it-6 #1019, SPEC-gitio-3/S-50: tree_with_paths joins real object/index operations.
use crate::fixture::{blob, commit, repo, set_ref, tree};
use crate::support::GitRepo;
use gitio::{Git, MergeTreeOutcome, Oid, RunOpts};
use std::{ffi::OsString, fs, os::unix::ffi::OsStringExt, path::{Path, PathBuf}};
use testkit::write_git_shim;

#[test]
fn selected_changes_additions_deletions_modes_and_objects_leave_base_elsewhere_and_dirty_folder_intact() {
    let r = repo();
    let base = tree(&r.git, &[("conflict", b"base\n"), ("gone", b"remove\n"), ("outside", b"zone work\n")]);
    let head = commit(&r.git, &base, &[], "base\n");
    set_ref(&r.git, "refs/heads/main", &head);
    run(&r.git, &["checkout", "--force", "main"]);
    fs::write(r.path.join("conflict"), b"staged local bytes\n").unwrap();
    run(&r.git, &["add", "--", "conflict"]);
    fs::write(r.path.join("conflict"), b"unstaged local bytes\n").unwrap();
    fs::write(r.path.join("untracked"), b"local\0\xff").unwrap();
    let from = mode_tree(&r.git, &[
        ("100755", "conflict", b"resolved\n"), ("100644", "added", b"added\n"),
        ("120000", "link", b"../destination"), ("100644", "outside", b"must not replace zone\n"),
    ], Some(("module", &head)));
    let expected = mode_tree(&r.git, &[
        ("100755", "conflict", b"resolved\n"), ("100644", "added", b"added\n"),
        ("120000", "link", b"../destination"), ("100644", "outside", b"zone work\n"),
    ], Some(("module", &head)));
    let before = observe(&r);
    let (git, trace, _) = traced(&r, false);
    let result = git.tree_with_paths(&base, &from, &paths(&["conflict", "added", "gone", "link", "module", "absent-both"])).unwrap();
    assert_eq!(result, expected, "literal final entries include executable, symlink and gitlink modes");
    assert_eq!(blob(&r.git, &result, "conflict"), b"resolved\n");
    assert_eq!(blob(&r.git, &base, "conflict"), b"base\n");
    assert_eq!(blob(&r.git, &result, "outside"), b"zone work\n");
    assert_eq!(observe(&r), before);
    assert_cleaned(&trace);
}

#[test]
fn colon_newline_tab_and_decomposed_unicode_paths_are_literal_not_patterns_or_precomposed() {
    let r = repo();
    run(&r.git, &["config", "core.precomposeunicode", "true"]);
    let base = tree(&r.git, &[(":(glob)*", b"old\n"), ("e\u{301}", b"old decomposed\n"), ("é", b"keep composed\n"), ("elsewhere", b"keep\n")]);
    let from = tree(&r.git, &[(":(glob)*", b"literal colon\n"), ("line\nbreak\tname", b"literal controls\n"), ("e\u{301}", b"decomposed\n"), ("é", b"not selected\n")]);
    let expected = tree(&r.git, &[(":(glob)*", b"literal colon\n"), ("line\nbreak\tname", b"literal controls\n"), ("e\u{301}", b"decomposed\n"), ("é", b"keep composed\n"), ("elsewhere", b"keep\n")]);
    let result = r.git.tree_with_paths(&base, &from, &paths(&[":(glob)*", "line\nbreak\tname", "e\u{301}"])).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn file_to_folder_entry_and_folder_to_file_replace_only_selected_area() {
    let r = repo();
    let file = tree(&r.git, &[("swap", b"file\n"), ("outside", b"keep\n")]);
    let folder = tree(&r.git, &[("swap/child", b"child\n"), ("outside", b"keep\n")]);
    assert_eq!(r.git.tree_with_paths(&file, &folder, &paths(&["swap/child"])).unwrap(), folder);
    assert_eq!(r.git.tree_with_paths(&folder, &file, &paths(&["swap"])).unwrap(), file);
}

#[test]
fn rename_rename_conflict_missing_original_path_can_be_deleted_without_reverting_clean_zone_change() {
    // Real merge_tree -> tree_with_paths; rename/rename's original path is conflicted but absent from the tree.
    let r = repo();
    let base_tree = tree(&r.git, &[("original", b"identical rename payload\n"), ("zone", b"base\n")]);
    let base = commit(&r.git, &base_tree, &[], "base\n");
    let ours_tree = tree(&r.git, &[("ours", b"identical rename payload\n"), ("zone", b"zone advanced\n")]);
    let ours = commit(&r.git, &ours_tree, &[base.clone()], "ours\n");
    let theirs_tree = tree(&r.git, &[("theirs", b"identical rename payload\n"), ("zone", b"base\n")]);
    let theirs = commit(&r.git, &theirs_tree, &[base.clone()], "theirs\n");
    let MergeTreeOutcome::Conflicted { tree: conflicted, files } = r.git.merge_tree(&base, None, &ours, &theirs).unwrap() else { panic!("expected real rename/rename conflict") };
    assert!(files.contains(&PathBuf::from("original")));
    assert_eq!(r.git.tree_entry(&conflicted, Path::new("original")).unwrap(), None);
    let caller = tree(&r.git, &[("ours", b"caller resolution\n"), ("zone", b"stale caller\n")]);
    let expected = tree(&r.git, &[("ours", b"caller resolution\n"), ("zone", b"zone advanced\n")]);
    assert_eq!(r.git.tree_with_paths(&conflicted, &caller, &files).unwrap(), expected);
}

#[test]
fn all_five_refusal_classes_precede_any_temp_index_command_and_leave_folder_unchanged() {
    let r = repo();
    let base = tree(&r.git, &[("base-dir/child", b"base\n"), ("ordinary", b"base\n")]);
    let from = tree(&r.git, &[("from-dir/child", b"from\n"), ("ordinary", b"from\n")]);
    let head = commit(&r.git, &base, &[], "base\n");
    set_ref(&r.git, "refs/heads/main", &head);
    run(&r.git, &["checkout", "--force", "main"]);
    let before = observe(&r);
    let (git, trace, calls) = traced(&r, false);
    let mut cases = vec![
        ("folder in from", paths(&["from-dir"])),
        ("missing in from but folder in base", paths(&["base-dir"])),
        ("overlapping paths", paths(&["ordinary/child", "ordinary"])),
    ];
    for bad in ["/ordinary", "ordinary/", "a//b", "a/./b", "a/../b", "./ordinary"] {
        cases.push(("not stored spelling", vec![PathBuf::from(bad)]));
    }
    cases.push(("empty", paths(&[""])));
    cases.push(("NUL", vec![PathBuf::from(OsString::from_vec(b"has\0nul".to_vec()))]));
    for (class, selected) in cases {
        let error = git.tree_with_paths(&base, &from, &selected).expect_err(class);
        assert_eq!(error.code, None, "{class}: {error:?}");
        assert!(!error.stderr.is_empty(), "{class}: refusal must explain the invalid path");
        assert!(!trace.exists(), "{class}: a temporary index was submitted to git");
        let commands = fs::read_to_string(&calls).unwrap_or_default();
        assert!(!commands.contains("read-tree") && !commands.contains("update-index") && !commands.contains("write-tree"), "{class}: {commands}");
        assert_eq!(observe(&r), before, "{class}");
    }
}

#[test]
fn empty_paths_return_base_tree_and_cleanup_also_runs_after_a_post_read_tree_error() {
    let r = repo();
    let base = tree(&r.git, &[("file", b"base\n")]);
    let from = tree(&r.git, &[("file", b"from\n")]);
    let (git, trace, _) = traced(&r, false);
    assert_eq!(git.tree_with_paths(&base, &from, &[]).unwrap(), base);
    assert_cleaned(&trace);
    let (failed, failed_trace, _) = traced(&r, true);
    let error = failed.tree_with_paths(&base, &from, &paths(&["file"])).unwrap_err();
    assert_eq!(error.code, Some(73));
    assert!(error.stderr.contains("fixture write-tree refusal"));
    assert_cleaned(&failed_trace);
    let missing = Oid::parse(&"0".repeat(40)).unwrap();
    assert!(git.tree_with_paths(&missing, &from, &[]).is_err(), "empty selection still validates base through real git");
    assert_cleaned(&trace);
}

fn paths(names: &[&str]) -> Vec<PathBuf> { names.iter().map(PathBuf::from).collect() }
fn run(git: &Git, args: &[&str]) -> Vec<u8> { git.run(args, &RunOpts::default()).unwrap().stdout }

fn mode_tree(git: &Git, files: &[(&str, &str, &[u8])], gitlink: Option<(&str, &Oid)>) -> Oid {
    let opts = RunOpts { env: vec![("GIT_INDEX_FILE".into(), git.repo.join(".git/mode-fixture-index").into_os_string())], ..RunOpts::default() };
    git.run(&["read-tree", "--empty"], &opts).unwrap();
    for (mode, path, bytes) in files {
        let id = git.run(&["hash-object", "-w", "--stdin"], &RunOpts { stdin: Some(bytes.to_vec()), ..RunOpts::default() }).unwrap().stdout;
        git.run(&["update-index", "--add", "--cacheinfo", mode, std::str::from_utf8(&id).unwrap().trim_end(), path], &opts).unwrap();
    }
    if let Some((path, id)) = gitlink { git.run(&["update-index", "--add", "--cacheinfo", "160000", id.as_str(), path], &opts).unwrap(); }
    let out = git.run(&["write-tree"], &opts).unwrap().stdout;
    Oid::parse(std::str::from_utf8(&out).unwrap().trim_end()).unwrap()
}

fn traced(r: &GitRepo, fail: bool) -> (Git, PathBuf, PathBuf) {
    let suffix = if fail { "failure" } else { "success" };
    let shim = r.path.join(format!(".git/tree-paths-{suffix}.sh"));
    let trace = r.path.join(format!(".git/tree-paths-{suffix}.index"));
    let calls = r.path.join(format!(".git/tree-paths-{suffix}.calls"));
    write_git_shim(&shim, br#"#!/bin/sh
set -eu
printf '%s\n' "$*" >> "$MORTAR_TEST_PATHS_CALLS"
if [ -n "${GIT_INDEX_FILE-}" ]; then printf '%s' "$GIT_INDEX_FILE" > "$MORTAR_TEST_PATHS_INDEX"; fi
if [ "$MORTAR_TEST_PATHS_FAIL" = 1 ] && [ "${3-}" = write-tree ]; then
  test -s "$GIT_INDEX_FILE"
  printf 'fixture write-tree refusal\n' >&2
  exit 73
fi
exec git "$@"
"#).unwrap();
    let mut git = r.git.clone();
    git.program = shim;
    git.env.extend([
        ("MORTAR_TEST_PATHS_CALLS".into(), calls.clone().into_os_string()),
        ("MORTAR_TEST_PATHS_INDEX".into(), trace.clone().into_os_string()),
        ("MORTAR_TEST_PATHS_FAIL".into(), if fail { "1" } else { "0" }.into()),
    ]);
    (git, trace, calls)
}

fn assert_cleaned(trace: &Path) {
    let index = PathBuf::from(fs::read_to_string(trace).unwrap());
    assert!(!index.as_os_str().is_empty());
    assert!(!index.exists(), "temporary index survived: {}", index.display());
    assert!(!PathBuf::from(format!("{}.lock", index.display())).exists());
}

#[derive(Debug, PartialEq, Eq)]
struct Observed { index: Vec<u8>, refs: Vec<u8>, head: Vec<u8>, files: Vec<(PathBuf, Option<Vec<u8>>)> }
fn observe(r: &GitRepo) -> Observed {
    let mut files = Vec::new();
    walk(&r.path, &r.path, &mut files);
    files.sort();
    Observed { index: fs::read(r.path.join(".git/index")).unwrap(), refs: run(&r.git, &["for-each-ref", "--format=%(refname) %(objectname)"]), head: fs::read(r.path.join(".git/HEAD")).unwrap(), files }
}
fn walk(root: &Path, dir: &Path, out: &mut Vec<(PathBuf, Option<Vec<u8>>)>) {
    for item in fs::read_dir(dir).unwrap() {
        let item = item.unwrap(); let path = item.path();
        if path == root.join(".git") { continue; }
        let relative = path.strip_prefix(root).unwrap().to_path_buf();
        if item.file_type().unwrap().is_dir() { out.push((relative, None)); walk(root, &path, out); }
        else { out.push((relative, Some(fs::read(path).unwrap()))); }
    }
}
