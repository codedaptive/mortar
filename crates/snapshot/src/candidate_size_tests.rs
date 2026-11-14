use crate::candidate_size::candidate_size;
use std::io::{Error, ErrorKind};

#[test]
fn a_file_or_symlink_gives_its_length() {
    assert_eq!(candidate_size(Ok((false, 7))).unwrap(), Some(7));
}

#[test]
fn a_directory_gives_none() {
    assert_eq!(candidate_size(Ok((true, 96))).unwrap(), None);
}

#[test]
fn a_path_not_found_gives_none() {
    assert_eq!(candidate_size(Err(Error::from(ErrorKind::NotFound))).unwrap(), None);
}

#[test]
fn any_other_error_passes_through() {
    let error = candidate_size(Err(Error::from(ErrorKind::PermissionDenied))).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::PermissionDenied);
}
