use crate::path_line::path_line;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

#[test]
fn absolute_line_is_the_path() {
    assert_eq!(path_line(b"/tmp/wo-gitio-2b-capture/.git\n").unwrap(), Path::new("/tmp/wo-gitio-2b-capture/.git"));
}

#[test]
fn only_one_newline_is_removed() {
    assert_eq!(path_line(b"/tmp/odd\n\n").unwrap().as_os_str().as_bytes(), b"/tmp/odd\n");
}

#[test]
fn line_without_newline_is_the_path() {
    assert_eq!(path_line(b"/tmp/r").unwrap(), Path::new("/tmp/r"));
}

#[test]
fn non_utf8_path_is_kept() {
    assert_eq!(path_line(b"/tmp/caf\xe9\n").unwrap().as_os_str().as_bytes(), b"/tmp/caf\xe9");
}

#[test]
fn relative_path_is_refused_naming_it() {
    let err = path_line(b".git\n").unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert!(err.stderr.contains("\".git\\n\""), "{}", err.stderr);
}

#[test]
fn empty_output_is_refused() {
    assert!(path_line(b"").is_err());
}

#[test]
fn bare_newline_is_refused() {
    assert!(path_line(b"\n").is_err());
}
