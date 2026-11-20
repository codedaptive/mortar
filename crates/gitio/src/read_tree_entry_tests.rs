use crate::read_tree_entry::read_tree_entry;
use crate::{EntryKind, TreeEntryError};
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

fn found(stdout: &[u8], path: &str) -> String {
    read_tree_entry(stdout, Path::new(path)).unwrap().unwrap().as_str().to_string()
}

fn refused(stdout: &[u8], path: &str) -> String {
    let TreeEntryError::Git(err) = read_tree_entry(stdout, Path::new(path)).unwrap_err() else {
        panic!("not refused as Git");
    };
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    err.stderr
}

fn not_a_blob(path: &str, kind: EntryKind) -> TreeEntryError {
    TreeEntryError::NotABlob { path: PathBuf::from(path), kind }
}

#[test]
fn captured_blob_is_its_id() {
    assert_eq!(
        found(b"100644 blob 5626abf0f72e58d7a153368ba57db4c673c0e171\tf\x00", "f"),
        "5626abf0f72e58d7a153368ba57db4c673c0e171"
    );
}

#[test]
fn captured_symlink_is_its_blob_id() {
    assert_eq!(
        found(b"120000 blob 4d1ae35ba2c8ec712fa2a379db44ad639ca277bd\tlink\x00", "link"),
        "4d1ae35ba2c8ec712fa2a379db44ad639ca277bd"
    );
}

#[test]
fn captured_path_holding_a_newline_is_found() {
    assert_eq!(
        found(b"100644 blob bec81d2b1ca4cdf376a684e3483bcfd13965916e\tn\nl\x00", "n\nl"),
        "bec81d2b1ca4cdf376a684e3483bcfd13965916e"
    );
}

#[test]
fn captured_dash_path_is_found() {
    assert_eq!(
        found(b"100644 blob 4bcfe98e640c8284511312660fb8709b0afa888e\t-dash\x00", "-dash"),
        "4bcfe98e640c8284511312660fb8709b0afa888e"
    );
}

#[test]
fn a_name_starting_with_a_colon_is_found() {
    assert_eq!(
        found(b"100644 blob 5626abf0f72e58d7a153368ba57db4c673c0e171\t:x\x00", ":x"),
        "5626abf0f72e58d7a153368ba57db4c673c0e171"
    );
}

#[test]
fn a_path_holding_a_tab_is_found() {
    assert_eq!(
        found(b"100644 blob 4bcfe98e640c8284511312660fb8709b0afa888e\tt\tb\x00", "t\tb"),
        "4bcfe98e640c8284511312660fb8709b0afa888e"
    );
}

#[test]
fn a_non_utf8_path_is_found_byte_for_byte() {
    let stdout = b"100644 blob 4bcfe98e640c8284511312660fb8709b0afa888e\tcaf\xe9\x00";
    let oid = read_tree_entry(stdout, Path::new(OsStr::from_bytes(b"caf\xe9"))).unwrap().unwrap();
    assert_eq!(oid.as_str(), "4bcfe98e640c8284511312660fb8709b0afa888e");
}

#[test]
fn sha256_blob_is_its_id() {
    let stdout = b"100644 blob 0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\tf\x00";
    assert_eq!(found(stdout, "f"), "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef");
}

#[test]
fn empty_output_is_a_missing_path() {
    assert_eq!(read_tree_entry(b"", Path::new("nosuch")), Ok(None));
}

#[test]
fn captured_tree_entry_is_refused_as_not_a_blob() {
    let stdout = b"040000 tree aaff74984cccd156a469afa7d9ab10e4777beb24\tdir\x00";
    assert_eq!(read_tree_entry(stdout, Path::new("dir")), Err(not_a_blob("dir", EntryKind::Tree)));
}

#[test]
fn a_submodule_commit_is_refused_as_not_a_blob() {
    let stdout = b"160000 commit aaff74984cccd156a469afa7d9ab10e4777beb24\tsub\x00";
    assert_eq!(read_tree_entry(stdout, Path::new("sub")), Err(not_a_blob("sub", EntryKind::Commit)));
}

#[test]
fn an_unknown_type_is_refused_as_not_a_blob_kept_as_found() {
    let stdout = b"100644 t\xffg aaff74984cccd156a469afa7d9ab10e4777beb24\tx\x00";
    assert_eq!(
        read_tree_entry(stdout, Path::new("x")),
        Err(not_a_blob("x", EntryKind::Other("t\u{fffd}g".to_string())))
    );
}

#[test]
fn a_bad_id_is_refused_before_the_type() {
    let stderr = refused(b"040000 tree AAFF74984CCCD156A469AFA7D9AB10E4777BEB24\tdir\x00", "dir");
    assert!(stderr.contains("(a bad object id)"), "{stderr}");
}

#[test]
fn folder_children_record_is_refused_as_another_path() {
    let stderr = refused(b"100644 blob 78981922613b2afb6025042ff6bd878ac1994e85\tdir/a\x00", "dir/");
    assert_eq!(
        stderr,
        "not one ls-tree record for \"dir/\" (another path): \
         \"100644 blob 78981922613b2afb6025042ff6bd878ac1994e85\\tdir/a\\0\""
    );
}
