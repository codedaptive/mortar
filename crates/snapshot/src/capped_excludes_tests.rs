use crate::capped_excludes::capped_excludes;
use crate::refused_file::RefusedFile;
use crate::snapshot_error::SnapshotError;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

fn refused(path: PathBuf) -> RefusedFile {
    RefusedFile { path, size: 200, cap: 100 }
}

#[test]
fn no_refused_files_gives_extra_unchanged() {
    let extra = vec!["dist/".to_string(), "*.log".to_string()];
    assert_eq!(capped_excludes(&extra, &[]).unwrap(), vec!["dist/", "*.log"]);
}

#[test]
fn refused_paths_follow_extra_as_literals_in_order() {
    let extra = vec!["dist/".to_string()];
    let refused = [refused(PathBuf::from("big.bin")), refused(PathBuf::from("a*b.bin"))];
    assert_eq!(capped_excludes(&extra, &refused).unwrap(), vec!["dist/", r":\big.bin", r":\a\*b.bin"]);
}

#[test]
fn a_refused_path_not_utf8_gives_path_not_utf8_naming_it() {
    let bad = PathBuf::from(OsStr::from_bytes(b"a\xffb"));
    match capped_excludes(&[], &[refused(PathBuf::from("ok.bin")), refused(bad)]) {
        Err(SnapshotError::PathNotUtf8 { path }) => assert_eq!(path, PathBuf::from(OsStr::from_bytes(b"a\xffb"))),
        other => panic!("expected PathNotUtf8, got {other:?}"),
    }
}
