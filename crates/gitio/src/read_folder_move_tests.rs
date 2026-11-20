use crate::read_folder_move::read_folder_move;
use crate::{FolderMoveOutcome, GitError, Output};
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

fn output(status: i32, stdout: &[u8], stderr: &[u8]) -> Output {
    Output {
        status,
        stdout: stdout.to_vec(),
        stderr: stderr.to_vec(),
    }
}

fn refused(stderr: &[u8]) -> Result<FolderMoveOutcome, GitError> {
    read_folder_move(&output(128, b"", stderr))
}

fn paths(items: &[&[u8]]) -> FolderMoveOutcome {
    FolderMoveOutcome::Refused(items.iter().map(|p| PathBuf::from(OsStr::from_bytes(p))).collect())
}

fn gits_error(stderr: &str) -> Result<FolderMoveOutcome, GitError> {
    Err(GitError {
        args: Vec::new(),
        code: Some(128),
        stderr: stderr.to_string(),
    })
}

#[test]
fn exit_0_with_nothing_written_is_moved() {
    assert_eq!(read_folder_move(&output(0, b"", b"")), Ok(FolderMoveOutcome::Moved));
}

#[test]
fn a_local_edit_in_the_way_is_refused_naming_it() {
    assert_eq!(refused(b"error: Entry 'f' not uptodate. Cannot merge.\n"), Ok(paths(&[b"f"])));
}

#[test]
fn a_staged_change_in_the_way_is_refused_naming_it() {
    assert_eq!(refused(b"error: Entry 'f' would be overwritten by merge. Cannot merge.\n"), Ok(paths(&[b"f"])));
}

#[test]
fn an_untracked_file_in_the_way_is_refused_naming_it() {
    assert_eq!(refused(b"error: Untracked working tree file 'new' would be overwritten by merge.\n"), Ok(paths(&[b"new"])));
}

#[test]
fn two_refusal_records_give_both_paths_in_order() {
    let stderr = b"error: Untracked working tree file 'd/new' would be overwritten by merge.\nerror: Entry 'f' not uptodate. Cannot merge.\n";
    assert_eq!(refused(stderr), Ok(paths(&[b"d/new", b"f"])));
}

#[test]
fn a_path_holding_a_quote_and_a_newline_is_kept_whole() {
    let stderr = b"error: Entry 'it's\nx' would be overwritten by merge. Cannot merge.\nerror: Entry 'g' not uptodate. Cannot merge.\n";
    assert_eq!(refused(stderr), Ok(paths(&[b"it's\nx", b"g"])));
}

#[test]
fn an_entry_record_ends_only_at_an_entry_tail() {
    let stderr = b"error: Entry 'a' would be overwritten by merge.\nb' not uptodate. Cannot merge.\n";
    assert_eq!(refused(stderr), Ok(paths(&[b"a' would be overwritten by merge.\nb"])));
}

#[test]
fn a_path_holding_its_own_tail_is_cut_short_and_read_as_gits_error() {
    let stderr = "error: Entry 'a' not uptodate. Cannot merge.\nb' not uptodate. Cannot merge.\n";
    assert_eq!(refused(stderr.as_bytes()), gits_error(stderr));
}

#[test]
fn an_unmerged_index_is_gits_error_with_its_stderr() {
    assert_eq!(
        refused(b"fatal: You need to resolve your current index first\n"),
        gits_error("fatal: You need to resolve your current index first\n")
    );
}

#[test]
fn an_unknown_tree_is_gits_error_with_its_stderr() {
    assert_eq!(refused(b"fatal: Not a valid object name bad\n"), gits_error("fatal: Not a valid object name bad\n"));
}

#[test]
fn a_refusal_record_followed_by_other_text_is_gits_error() {
    let stderr = "error: Entry 'f' not uptodate. Cannot merge.\nfatal: something else\n";
    assert_eq!(refused(stderr.as_bytes()), gits_error(stderr));
}

#[test]
fn a_refusal_record_without_its_newline_is_gits_error() {
    let stderr = "error: Entry 'f' not uptodate. Cannot merge.";
    assert_eq!(refused(stderr.as_bytes()), gits_error(stderr));
}

#[test]
fn exit_128_with_empty_stderr_is_gits_error() {
    assert_eq!(refused(b""), gits_error(""));
}

#[test]
fn exit_128_with_stdout_is_gits_error_even_over_a_refusal_record() {
    let stderr = "error: Entry 'f' not uptodate. Cannot merge.\n";
    assert_eq!(read_folder_move(&output(128, b"x\n", stderr.as_bytes())), gits_error(stderr));
}

#[test]
fn exit_0_with_stdout_is_refused_naming_it() {
    let err = read_folder_move(&output(0, b"odd\n", b"")).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert_eq!(err.stderr, "read-tree answered exit 0 with stdout \"odd\\n\"");
}

#[test]
fn another_exit_is_refused_naming_it() {
    let err = read_folder_move(&output(1, b"", b"error: Entry 'f' not uptodate. Cannot merge.\n")).unwrap_err();
    assert_eq!(err.code, None);
    assert_eq!(err.stderr, "read-tree answered exit 1 with stdout \"\"");
}
