use crate::usable_path::usable_path;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

#[test]
fn plain_spaced_dashed_newline_and_glob_paths_pass() {
    for path in ["a.txt", "sp ace", "-dash.txt", "n\nl.txt", "d*", "./a.txt", "a/b/c"] {
        assert_eq!(usable_path(Path::new(path)), Ok(()), "{path:?}");
    }
}

#[test]
fn a_non_utf8_path_passes() {
    assert_eq!(usable_path(Path::new(OsStr::from_bytes(b"caf\xe9"))), Ok(()));
}

#[test]
fn empty_path_is_refused() {
    let err = usable_path(Path::new("")).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert_eq!(err.stderr, "not a usable path (empty): \"\"");
}

#[test]
fn nul_is_refused_naming_the_path() {
    let err = usable_path(Path::new(OsStr::from_bytes(b"a\0b"))).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert_eq!(err.stderr, "path holds a NUL: \"a\\0b\"");
}
