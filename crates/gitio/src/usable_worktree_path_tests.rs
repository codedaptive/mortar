use crate::usable_worktree_path::usable_worktree_path;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

#[test]
fn relative_path_passes() {
    assert_eq!(usable_worktree_path(Path::new("../wt1")), Ok(()));
}

#[test]
fn non_utf8_path_passes() {
    assert_eq!(usable_worktree_path(Path::new(OsStr::from_bytes(b"/tmp/caf\xe9"))), Ok(()));
}

#[test]
fn empty_path_is_refused() {
    let err = usable_worktree_path(Path::new("")).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert_eq!(err.stderr, "not a usable worktree path (empty): \"\"");
}

#[test]
fn nul_in_path_is_refused_naming_it() {
    let err = usable_worktree_path(Path::new(OsStr::from_bytes(b"../w\0t"))).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert_eq!(err.stderr, "worktree path holds a NUL: \"../w\\0t\"");
}
