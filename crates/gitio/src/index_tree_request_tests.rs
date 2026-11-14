use crate::index_tree_request::index_tree_request;
use crate::RunOpts;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

fn argv(items: &[&str]) -> Vec<OsString> {
    items.iter().map(OsString::from).collect()
}

fn requests(folder: &Path) -> [(Vec<OsString>, RunOpts); 3] {
    index_tree_request(folder, Path::new("/tmp/copy.index")).unwrap()
}

fn on_copy(copy: &[u8]) -> Vec<(OsString, OsString)> {
    vec![(OsString::from("GIT_INDEX_FILE"), OsStr::from_bytes(copy).to_os_string())]
}

#[test]
fn first_run_asks_git_for_the_folders_index_path_absolute() {
    assert_eq!(
        requests(Path::new("/tmp/r/wt"))[0].0,
        argv(&["rev-parse", "--path-format=absolute", "--git-path", "index"])
    );
}

#[test]
fn index_lookup_is_a_read_in_the_folder_with_no_env_accepting_only_exit_0() {
    let [(_, opts), _, _] = requests(Path::new("/tmp/r/wt"));
    assert_eq!(opts.cwd, Some(PathBuf::from("/tmp/r/wt")));
    assert!(opts.read_only);
    assert!(opts.answer_codes.is_empty());
    assert!(opts.env.is_empty());
    assert_eq!(opts.stdin, None);
}

#[test]
fn second_run_is_write_tree() {
    assert_eq!(requests(Path::new("/tmp/r/wt"))[1].0, argv(&["write-tree"]));
}

#[test]
fn write_tree_writes_in_the_folder_on_the_copy_and_accepts_only_exit_0() {
    let [_, (_, opts), _] = requests(Path::new("/tmp/r/wt"));
    assert_eq!(opts.cwd, Some(PathBuf::from("/tmp/r/wt")));
    assert!(!opts.read_only);
    assert!(opts.answer_codes.is_empty());
    assert_eq!(opts.env, on_copy(b"/tmp/copy.index"));
    assert_eq!(opts.stdin, None);
}

#[test]
fn third_run_lists_unmerged_entries_nul_ended_with_full_paths_from_the_top() {
    assert_eq!(
        requests(Path::new("/tmp/r/wt"))[2].0,
        argv(&["ls-files", "--unmerged", "-z", "--full-name", "--", ":/"])
    );
}

#[test]
fn ls_files_is_a_read_in_the_folder_on_the_copy_accepting_only_exit_0() {
    let [_, _, (_, opts)] = requests(Path::new("/tmp/r/wt"));
    assert_eq!(opts.cwd, Some(PathBuf::from("/tmp/r/wt")));
    assert!(opts.read_only);
    assert!(opts.answer_codes.is_empty());
    assert_eq!(opts.env, on_copy(b"/tmp/copy.index"));
    assert_eq!(opts.stdin, None);
}

#[test]
fn a_non_utf8_copy_path_is_kept_as_its_bytes_in_both_envs() {
    let [_, (_, write), (_, list)] =
        index_tree_request(Path::new("/tmp/r/wt"), Path::new(OsStr::from_bytes(b"/tmp/caf\xe9.index"))).unwrap();
    assert_eq!(write.env, on_copy(b"/tmp/caf\xe9.index"));
    assert_eq!(list.env, on_copy(b"/tmp/caf\xe9.index"));
}

#[test]
fn a_non_utf8_folder_is_kept_as_its_bytes_in_all_three_runs() {
    for (_, opts) in requests(Path::new(OsStr::from_bytes(b"/tmp/caf\xe9"))) {
        assert_eq!(opts.cwd.unwrap().as_os_str().as_bytes(), b"/tmp/caf\xe9");
    }
}

#[test]
fn empty_folder_is_refused() {
    let err = index_tree_request(Path::new(""), Path::new("/tmp/copy.index")).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert_eq!(err.stderr, "not a usable worktree path (empty): \"\"");
}

#[test]
fn nul_in_folder_is_refused_naming_it() {
    let folder = Path::new(OsStr::from_bytes(b"/tmp/w\0t"));
    let err = index_tree_request(folder, Path::new("/tmp/copy.index")).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert_eq!(err.stderr, "worktree path holds a NUL: \"/tmp/w\\0t\"");
}
