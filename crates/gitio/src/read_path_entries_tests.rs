use crate::read_path_entries::read_path_entries;
use crate::Oid;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

fn paths(items: &[&str]) -> Vec<PathBuf> {
    items.iter().map(PathBuf::from).collect()
}

fn entry(mode: &str, hex: &str) -> Option<(String, Oid)> {
    Some((mode.to_string(), Oid::parse(hex).unwrap()))
}

fn refused(stdout: &[u8], items: &[&str]) -> String {
    let err = read_path_entries(stdout, &paths(items)).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    err.stderr
}

#[test]
fn a_captured_blob_is_its_mode_and_id() {
    let stdout = b"100644 blob 5626abf0f72e58d7a153368ba57db4c673c0e171\tf\x00";
    assert_eq!(
        read_path_entries(stdout, &paths(&["f"])),
        Ok(vec![entry("100644", "5626abf0f72e58d7a153368ba57db4c673c0e171")])
    );
}

#[test]
fn a_captured_symlink_is_taken() {
    let stdout = b"120000 blob 4d1ae35ba2c8ec712fa2a379db44ad639ca277bd\tlink\x00";
    assert_eq!(
        read_path_entries(stdout, &paths(&["link"])),
        Ok(vec![entry("120000", "4d1ae35ba2c8ec712fa2a379db44ad639ca277bd")])
    );
}

#[test]
fn a_captured_gitlink_is_taken() {
    let stdout = b"160000 commit aaff74984cccd156a469afa7d9ab10e4777beb24\tsub\x00";
    assert_eq!(
        read_path_entries(stdout, &paths(&["sub"])),
        Ok(vec![entry("160000", "aaff74984cccd156a469afa7d9ab10e4777beb24")])
    );
}

#[test]
fn a_path_without_a_record_is_none() {
    let stdout = b"100644 blob 5626abf0f72e58d7a153368ba57db4c673c0e171\tf\x00";
    assert_eq!(
        read_path_entries(stdout, &paths(&["gone", "f"])),
        Ok(vec![None, entry("100644", "5626abf0f72e58d7a153368ba57db4c673c0e171")])
    );
}

#[test]
fn items_follow_the_request_order_not_the_output_order() {
    let stdout = b"100644 blob 1111111111111111111111111111111111111111\ta\x00\
                   100755 blob 2222222222222222222222222222222222222222\tb\x00";
    assert_eq!(
        read_path_entries(stdout, &paths(&["b", "a"])),
        Ok(vec![
            entry("100755", "2222222222222222222222222222222222222222"),
            entry("100644", "1111111111111111111111111111111111111111"),
        ])
    );
}

#[test]
fn a_path_asked_twice_gets_its_record_twice() {
    let stdout = b"100644 blob 1111111111111111111111111111111111111111\ta\x00";
    assert_eq!(
        read_path_entries(stdout, &paths(&["a", "a"])),
        Ok(vec![
            entry("100644", "1111111111111111111111111111111111111111"),
            entry("100644", "1111111111111111111111111111111111111111"),
        ])
    );
}

#[test]
fn a_path_holding_a_tab_and_a_newline_is_found() {
    let stdout = b"100644 blob 4bcfe98e640c8284511312660fb8709b0afa888e\tt\tb\nl\x00";
    assert_eq!(
        read_path_entries(stdout, &paths(&["t\tb\nl"])),
        Ok(vec![entry("100644", "4bcfe98e640c8284511312660fb8709b0afa888e")])
    );
}

#[test]
fn a_non_utf8_path_is_found_byte_for_byte() {
    let stdout = b"100644 blob 4bcfe98e640c8284511312660fb8709b0afa888e\tcaf\xe9\x00";
    assert_eq!(
        read_path_entries(stdout, &[PathBuf::from(OsStr::from_bytes(b"caf\xe9"))]),
        Ok(vec![entry("100644", "4bcfe98e640c8284511312660fb8709b0afa888e")])
    );
}

#[test]
fn a_sha256_id_is_taken() {
    let stdout = b"100644 blob 0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\tf\x00";
    assert_eq!(
        read_path_entries(stdout, &paths(&["f"])),
        Ok(vec![entry("100644", "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")])
    );
}

#[test]
fn empty_output_is_none_for_every_path() {
    assert_eq!(read_path_entries(b"", &paths(&["a", "b"])), Ok(vec![None, None]));
}

#[test]
fn a_captured_folder_is_refused_naming_the_path() {
    let stdout = b"040000 tree aaff74984cccd156a469afa7d9ab10e4777beb24\tdir\x00";
    assert_eq!(refused(stdout, &["dir"]), "not a file entry: \"dir\" is a folder");
}

#[test]
fn a_record_for_a_path_not_asked_is_refused_naming_the_output() {
    let stdout = b"100644 blob 78981922613b2afb6025042ff6bd878ac1994e85\tdir/a\x00";
    assert_eq!(
        refused(stdout, &["dir"]),
        "not ls-tree records for the paths asked (a path not asked): \
         \"100644 blob 78981922613b2afb6025042ff6bd878ac1994e85\\tdir/a\\0\""
    );
}

#[test]
fn a_mode_that_is_not_octal_is_refused() {
    let stderr = refused(b"100648 blob 5626abf0f72e58d7a153368ba57db4c673c0e171\tf\x00", &["f"]);
    assert!(stderr.contains("(a mode that is not octal digits)"), "{stderr}");
    assert!(stderr.contains("100648 blob"), "{stderr}");
}

#[test]
fn an_empty_mode_is_refused() {
    let stderr = refused(b" blob 5626abf0f72e58d7a153368ba57db4c673c0e171\tf\x00", &["f"]);
    assert!(stderr.contains("(a mode that is not octal digits)"), "{stderr}");
}

#[test]
fn a_bad_object_id_is_refused() {
    let stderr = refused(b"100644 blob 5626ABF0F72E58D7A153368BA57DB4C673C0E171\tf\x00", &["f"]);
    assert!(stderr.contains("(a bad object id)"), "{stderr}");
    assert!(stderr.contains("5626ABF0F72E58D7A153368BA57DB4C673C0E171"), "{stderr}");
}

#[test]
fn a_missing_final_nul_is_refused() {
    let stderr = refused(b"100644 blob 5626abf0f72e58d7a153368ba57db4c673c0e171\tf", &["f"]);
    assert!(stderr.contains("(no final NUL)"), "{stderr}");
}

#[test]
fn a_record_without_a_tab_is_refused() {
    let stderr = refused(b"100644 blob 5626abf0f72e58d7a153368ba57db4c673c0e171 f\x00", &["f"]);
    assert!(stderr.contains("(not <mode> <type> <oid>\\t<path>)"), "{stderr}");
}

#[test]
fn a_header_with_a_missing_or_extra_field_is_refused() {
    for stdout in [
        &b"blob 5626abf0f72e58d7a153368ba57db4c673c0e171\tf\x00"[..],
        &b"100644 blob 5626abf0f72e58d7a153368ba57db4c673c0e171 9\tf\x00"[..],
    ] {
        let stderr = refused(stdout, &["f"]);
        assert!(stderr.contains("(not <mode> <type> <oid>\\t<path>)"), "{stderr}");
    }
}

#[test]
fn another_type_is_refused() {
    let stderr = refused(b"100644 tag 5626abf0f72e58d7a153368ba57db4c673c0e171\tf\x00", &["f"]);
    assert!(stderr.contains("(a type that is not blob, commit or tree)"), "{stderr}");
}

#[test]
fn a_lone_nul_is_refused() {
    assert!(read_path_entries(b"\x00", &paths(&["f"])).is_err());
}
