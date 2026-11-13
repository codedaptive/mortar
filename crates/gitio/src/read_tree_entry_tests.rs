use crate::read_tree_entry::read_tree_entry;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

fn found(stdout: &[u8], path: &str) -> String {
    read_tree_entry(stdout, Path::new(path)).unwrap().unwrap().as_str().to_string()
}

fn refused(stdout: &[u8], path: &str) -> String {
    let err = read_tree_entry(stdout, Path::new(path)).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    err.stderr
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
    let stderr = refused(b"040000 tree aaff74984cccd156a469afa7d9ab10e4777beb24\tdir\x00", "dir");
    assert_eq!(stderr, "not a blob: \"dir\" is a \"tree\"");
}

#[test]
fn a_submodule_commit_is_refused_as_not_a_blob() {
    let stderr = refused(b"160000 commit aaff74984cccd156a469afa7d9ab10e4777beb24\tsub\x00", "sub");
    assert_eq!(stderr, "not a blob: \"sub\" is a \"commit\"");
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

#[test]
fn two_records_are_refused_naming_the_output() {
    let stdout = b"100644 blob 5626abf0f72e58d7a153368ba57db4c673c0e171\tf\x00\
                   120000 blob 4d1ae35ba2c8ec712fa2a379db44ad639ca277bd\tlink\x00";
    let stderr = refused(stdout, "f");
    assert!(stderr.contains("not exactly one NUL-ended record"), "{stderr}");
    assert!(stderr.contains("\\tlink\\0\""), "{stderr}");
}

#[test]
fn a_record_without_its_nul_is_refused() {
    let stderr = refused(b"100644 blob 5626abf0f72e58d7a153368ba57db4c673c0e171\tf", "f");
    assert!(stderr.contains("not exactly one NUL-ended record"), "{stderr}");
}

#[test]
fn a_bad_object_id_is_refused() {
    let stderr = refused(b"100644 blob 5626ABF0F72E58D7A153368BA57DB4C673C0E171\tf\x00", "f");
    assert!(stderr.contains("(a bad object id)"), "{stderr}");
    assert!(stderr.contains("5626ABF0F72E58D7A153368BA57DB4C673C0E171"), "{stderr}");
}

#[test]
fn a_record_without_a_tab_is_refused() {
    let stderr = refused(b"100644 blob 5626abf0f72e58d7a153368ba57db4c673c0e171 f\x00", "f");
    assert!(stderr.contains("not <mode> <type> <oid>\\t<path>)"), "{stderr}");
}

#[test]
fn a_header_with_a_missing_field_is_refused() {
    let stderr = refused(b"blob 5626abf0f72e58d7a153368ba57db4c673c0e171\tf\x00", "f");
    assert!(stderr.contains("not <mode> <type> <oid>"), "{stderr}");
}

#[test]
fn a_header_with_an_extra_field_is_refused() {
    let stderr = refused(b"100644 blob 5626abf0f72e58d7a153368ba57db4c673c0e171 9\tf\x00", "f");
    assert!(stderr.contains("not <mode> <type> <oid>"), "{stderr}");
}

#[test]
fn a_lone_nul_is_refused() {
    assert!(read_tree_entry(b"\x00", Path::new("f")).is_err());
}
