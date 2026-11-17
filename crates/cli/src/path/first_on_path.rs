//! first_on_path: the program a shell would run for a bare name, found the way the shell looks it up (S-133 (1): `mr
//! guard install` asks which `mr` comes first). S-104: it reads the file system; its tests make their own files.

use std::ffi::OsStr;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

/// The first `<entry>/<name>` over the entries of `path` (a PATH value) that is a regular file with any execute bit
/// set, following symlinks. The path is returned as the entry names it, not canonicalized: the caller canonicalizes
/// when it compares. Empty entries, missing dirs, dangling links, directories and non-executables are skipped.
pub fn first_on_path(name: &str, path: &OsStr) -> Option<PathBuf> {
    std::env::split_paths(path)
        .filter(|entry| !entry.as_os_str().is_empty())
        .map(|entry| entry.join(name))
        .find(|candidate| {
            fs::metadata(candidate).is_ok_and(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
        })
}
