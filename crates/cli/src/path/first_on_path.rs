//! first_on_path: the program a shell would run for a bare name, found the way the shell looks it up (S-133 (1): `mr
//! guard install` asks which `mr` comes first), over path_entries' reading of PATH (S-147). The current directory is
//! passed in, so tests never change the process's shared one.
//! No sibling test (S-5, S-104): its job is reading files. path_entries_tests cover the reading of PATH (unset, empty,
//! empty and relative entries). Its real-file cases (order, execute bits, symlinks, directories, missing dirs, the
//! cwd for empty entries) are listed for Nagatha's RITs through `mr guard install` (S-169).

use std::ffi::OsStr;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use super::path_entries;

/// The first `<entry>/<name>` over path_entries(`path`, `cwd`) (an unset PATH, None, searching nothing), in order,
/// that is a regular file with any execute bit set, following symlinks. The path is returned as the entry names it,
/// not canonicalized: the caller canonicalizes when it compares. Missing dirs, dangling links, directories and
/// non-executables are skipped.
pub fn first_on_path(name: &str, path: Option<&OsStr>, cwd: &Path) -> Option<PathBuf> {
    path_entries(path, cwd).into_iter().map(|entry| entry.join(name)).find(|candidate| {
        fs::metadata(candidate).is_ok_and(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
    })
}
