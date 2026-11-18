//! temp_path: The name of the sibling temp file install writes before renaming it over its target.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The sibling of `path` named `.<file name>.<token as 16 lowercase hex digits>.mortar-install`.
pub fn temp_path(path: &Path, token: u64) -> PathBuf {
    let mut name = OsString::from(".");
    name.push(path.file_name().unwrap_or_default());
    name.push(format!(".{token:016x}.mortar-install"));
    path.with_file_name(name)
}
