//! file_markers: a folder is never read and has no markers, a missing file has none, a file's bytes give its marker
//! line numbers (written out by hand), and any other read error comes back.

use crate::file_markers::file_markers;
use std::io::{Error, ErrorKind};

#[test]
fn a_folder_is_not_read_and_has_no_markers() {
    let markers = file_markers(true, || panic!("a folder must not be read")).unwrap();
    assert_eq!(markers, Vec::<usize>::new());
}

#[test]
fn a_missing_file_has_no_markers() {
    let markers = file_markers(false, || Err(Error::from(ErrorKind::NotFound))).unwrap();
    assert_eq!(markers, Vec::<usize>::new());
}

#[test]
fn a_two_way_conflict_gives_its_marker_lines() {
    let markers = file_markers(false, || Ok(b"a\n<<<<<<< ours\nx\n=======\ny\n>>>>>>> theirs\n".to_vec())).unwrap();
    assert_eq!(markers, vec![2, 4, 6]);
}

#[test]
fn clean_bytes_have_no_markers() {
    let markers = file_markers(false, || Ok(b"a\nx\ny\n".to_vec())).unwrap();
    assert_eq!(markers, Vec::<usize>::new());
}

#[test]
fn another_read_error_is_returned() {
    let error = file_markers(false, || Err(Error::from(ErrorKind::PermissionDenied))).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::PermissionDenied);
}
