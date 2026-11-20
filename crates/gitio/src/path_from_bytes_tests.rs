use crate::path_from_bytes::path_from_bytes;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

#[test]
fn utf8_path_is_kept() {
    assert_eq!(path_from_bytes(b"/tmp/r/.git"), Path::new("/tmp/r/.git"));
}

#[test]
fn non_utf8_bytes_round_trip() {
    let bytes: &[u8] = b"/tmp/caf\xe9/\xff\xfe name";
    assert_eq!(path_from_bytes(bytes).as_os_str().as_bytes(), b"/tmp/caf\xe9/\xff\xfe name");
}

#[test]
fn every_single_byte_but_nul_round_trips() {
    for byte in 1u8..=255 {
        let bytes = [b'/', b'a', byte];
        assert_eq!(path_from_bytes(&bytes).as_os_str().as_bytes(), &bytes[..], "byte {byte}");
    }
}

#[test]
fn trailing_newline_is_not_removed() {
    assert_eq!(path_from_bytes(b"/tmp/r\n").as_os_str().as_bytes(), b"/tmp/r\n");
}
