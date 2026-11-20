use crate::read_nul_paths::read_nul_paths;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

fn read(stdout: &[u8]) -> Vec<Vec<u8>> {
    read_nul_paths(stdout).unwrap().iter().map(|p| p.as_os_str().as_bytes().to_vec()).collect()
}

#[test]
fn captured_paths_keep_newlines_and_spaces_in_order() {
    assert_eq!(
        read(b"nl\nname\0sp ace\0top\0top2\0"),
        vec![b"nl\nname".to_vec(), b"sp ace".to_vec(), b"top".to_vec(), b"top2".to_vec()]
    );
}

#[test]
fn one_path_is_one_entry() {
    assert_eq!(read_nul_paths(b"a/b/c.rs\0").unwrap(), vec![PathBuf::from("a/b/c.rs")]);
}

#[test]
fn a_path_named_like_an_oid_is_still_a_path() {
    assert_eq!(
        read_nul_paths(b"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\0").unwrap(),
        vec![PathBuf::from("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")]
    );
}

#[test]
fn non_utf8_path_is_kept() {
    assert_eq!(read(b"caf\xe9\0"), vec![b"caf\xe9".to_vec()]);
}

#[test]
fn empty_output_is_no_paths() {
    assert!(read_nul_paths(b"").unwrap().is_empty());
}

#[test]
fn last_path_without_nul_is_refused_naming_the_output() {
    let err = read_nul_paths(b"top\0top2").unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert!(err.stderr.contains("\"top\\0top2\""), "{}", err.stderr);
}

#[test]
fn empty_path_in_the_middle_is_refused() {
    let err = read_nul_paths(b"a\0\0b\0").unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.stderr.contains("empty path"), "{}", err.stderr);
}

#[test]
fn lone_nul_is_refused() {
    assert!(read_nul_paths(b"\0").is_err());
}
