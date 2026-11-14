use crate::read_tree_entry_record::read_tree_entry_record;
use std::path::Path;

fn found(stdout: &[u8], path: &str) -> (Vec<u8>, String) {
    let (kind, oid) = read_tree_entry_record(stdout, Path::new(path)).unwrap().unwrap();
    (kind.to_vec(), oid.as_str().to_string())
}

fn refused(stdout: &[u8], path: &str) -> String {
    let err = read_tree_entry_record(stdout, Path::new(path)).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    err.stderr
}

#[test]
fn captured_blob_record_is_its_type_and_id() {
    assert_eq!(
        found(b"100644 blob 5626abf0f72e58d7a153368ba57db4c673c0e171\tf\x00", "f"),
        (b"blob".to_vec(), "5626abf0f72e58d7a153368ba57db4c673c0e171".to_string())
    );
}

#[test]
fn captured_tree_record_is_its_type_and_id() {
    assert_eq!(
        found(b"040000 tree aaff74984cccd156a469afa7d9ab10e4777beb24\tdir\x00", "dir"),
        (b"tree".to_vec(), "aaff74984cccd156a469afa7d9ab10e4777beb24".to_string())
    );
}

#[test]
fn submodule_commit_record_is_its_type_and_id() {
    assert_eq!(
        found(b"160000 commit aaff74984cccd156a469afa7d9ab10e4777beb24\tsub\x00", "sub"),
        (b"commit".to_vec(), "aaff74984cccd156a469afa7d9ab10e4777beb24".to_string())
    );
}

#[test]
fn empty_output_is_a_missing_path() {
    assert_eq!(read_tree_entry_record(b"", Path::new("nosuch")), Ok(None));
}

#[test]
fn folder_children_record_is_refused_as_another_path() {
    assert_eq!(
        refused(b"100644 blob 78981922613b2afb6025042ff6bd878ac1994e85\tdir/a\x00", "dir/"),
        "not one ls-tree record for \"dir/\" (another path): \
         \"100644 blob 78981922613b2afb6025042ff6bd878ac1994e85\\tdir/a\\0\""
    );
}

#[test]
fn two_records_are_refused_naming_the_output() {
    let stdout = b"100644 blob 5626abf0f72e58d7a153368ba57db4c673c0e171\tf\x00\
                   120000 blob 4d1ae35ba2c8ec712fa2a379db44ad639ca277bd\tlink\x00";
    assert_eq!(
        refused(stdout, "f"),
        "not one ls-tree record for \"f\" (not exactly one NUL-ended record): \
         \"100644 blob 5626abf0f72e58d7a153368ba57db4c673c0e171\\tf\\0\
         120000 blob 4d1ae35ba2c8ec712fa2a379db44ad639ca277bd\\tlink\\0\""
    );
}

#[test]
fn a_record_without_its_nul_is_refused() {
    assert_eq!(
        refused(b"100644 blob 5626abf0f72e58d7a153368ba57db4c673c0e171\tf", "f"),
        "not one ls-tree record for \"f\" (not exactly one NUL-ended record): \
         \"100644 blob 5626abf0f72e58d7a153368ba57db4c673c0e171\\tf\""
    );
}

#[test]
fn a_bad_object_id_is_refused() {
    assert_eq!(
        refused(b"100644 blob 5626ABF0F72E58D7A153368BA57DB4C673C0E171\tf\x00", "f"),
        "not one ls-tree record for \"f\" (a bad object id): \
         \"100644 blob 5626ABF0F72E58D7A153368BA57DB4C673C0E171\\tf\\0\""
    );
}

#[test]
fn a_record_without_a_tab_is_refused() {
    assert_eq!(
        refused(b"100644 blob 5626abf0f72e58d7a153368ba57db4c673c0e171 f\x00", "f"),
        "not one ls-tree record for \"f\" (not <mode> <type> <oid>\\t<path>): \
         \"100644 blob 5626abf0f72e58d7a153368ba57db4c673c0e171 f\\0\""
    );
}

#[test]
fn a_header_with_a_missing_field_is_refused() {
    assert_eq!(
        refused(b"blob 5626abf0f72e58d7a153368ba57db4c673c0e171\tf\x00", "f"),
        "not one ls-tree record for \"f\" (not <mode> <type> <oid>\\t<path>): \
         \"blob 5626abf0f72e58d7a153368ba57db4c673c0e171\\tf\\0\""
    );
}

#[test]
fn a_header_with_an_extra_field_is_refused() {
    assert_eq!(
        refused(b"100644 blob 5626abf0f72e58d7a153368ba57db4c673c0e171 9\tf\x00", "f"),
        "not one ls-tree record for \"f\" (not <mode> <type> <oid>\\t<path>): \
         \"100644 blob 5626abf0f72e58d7a153368ba57db4c673c0e171 9\\tf\\0\""
    );
}

#[test]
fn a_lone_nul_is_refused() {
    assert_eq!(
        refused(b"\x00", "f"),
        "not one ls-tree record for \"f\" (not <mode> <type> <oid>\\t<path>): \"\\0\""
    );
}
