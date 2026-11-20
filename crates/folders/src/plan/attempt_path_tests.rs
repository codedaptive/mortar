//! Attempt folder paths inside the caller's directory.

use std::path::{Path, PathBuf};

use super::attempt_path;

#[test]
fn puts_attempt_k_at_try_k() {
    assert_eq!(attempt_path(Path::new("/w/r6-tries"), 1), PathBuf::from("/w/r6-tries/try-1"));
    assert_eq!(attempt_path(Path::new("/w/r6-tries"), 3), PathBuf::from("/w/r6-tries/try-3"));
}

#[test]
fn keeps_a_relative_directory_relative() {
    assert_eq!(attempt_path(Path::new("tries"), 10), PathBuf::from("tries/try-10"));
}
