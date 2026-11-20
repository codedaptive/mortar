use crate::read_path_kind::read_path_kind;
use crate::PathKind;
use std::path::Path;

fn kind(stdout: &[u8], path: &str) -> Option<PathKind> {
    read_path_kind(stdout, Path::new(path)).unwrap()
}

#[test]
fn a_regular_file_is_file() {
    assert_eq!(kind(b"100644 blob 5626abf0f72e58d7a153368ba57db4c673c0e171\tf\x00", "f"), Some(PathKind::File));
}

#[test]
fn a_symlink_is_file() {
    assert_eq!(
        kind(b"120000 blob 4d1ae35ba2c8ec712fa2a379db44ad639ca277bd\tlink\x00", "link"),
        Some(PathKind::File)
    );
}

#[test]
fn an_executable_is_file() {
    assert_eq!(
        kind(b"100755 blob 5626abf0f72e58d7a153368ba57db4c673c0e171\trun.sh\x00", "run.sh"),
        Some(PathKind::File)
    );
}

#[test]
fn a_tree_is_folder() {
    assert_eq!(
        kind(b"040000 tree aaff74984cccd156a469afa7d9ab10e4777beb24\tdir\x00", "dir"),
        Some(PathKind::Folder)
    );
}

#[test]
fn a_commit_is_gitlink() {
    assert_eq!(
        kind(b"160000 commit aaff74984cccd156a469afa7d9ab10e4777beb24\tsub\x00", "sub"),
        Some(PathKind::Gitlink)
    );
}

#[test]
fn empty_output_is_none() {
    assert_eq!(kind(b"", "nosuch"), None);
}

#[test]
fn a_name_starting_with_a_colon_is_file() {
    assert_eq!(
        kind(b"100644 blob 5626abf0f72e58d7a153368ba57db4c673c0e171\t:x\x00", ":x"),
        Some(PathKind::File)
    );
}

#[test]
fn a_name_holding_a_newline_is_file() {
    assert_eq!(
        kind(b"100644 blob 5626abf0f72e58d7a153368ba57db4c673c0e171\ta\nb\x00", "a\nb"),
        Some(PathKind::File)
    );
}

#[test]
fn an_unknown_type_is_refused_naming_it() {
    let err =
        read_path_kind(b"100644 tag 5626abf0f72e58d7a153368ba57db4c673c0e171\tf\x00", Path::new("f")).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert_eq!(err.stderr, "not a blob, tree or commit: \"f\" is a \"tag\"");
}

#[test]
fn two_records_are_refused() {
    let stdout = b"100644 blob 5626abf0f72e58d7a153368ba57db4c673c0e171\tf\x00\
                   040000 tree aaff74984cccd156a469afa7d9ab10e4777beb24\tdir\x00";
    let err = read_path_kind(stdout, Path::new("f")).unwrap_err();
    assert_eq!(err.code, None);
}
