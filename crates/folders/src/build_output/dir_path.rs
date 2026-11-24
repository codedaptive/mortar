//! DirFd off macOS: a directory named by its path, with the same methods as dir_fd.rs and no `unsafe`.
//! The looks are by path. Nothing is made or cloned off macOS: clonefile(2) is macOS only, so clone_build_dirs
//! refuses every clone on NO_CLONE before any make, and that refusal is the only result a clone has here.
//! make_dir and clone_into are there for the shared callers to compile; NO_CLONE stops every clone before them.
//! No unit test: it only reads the filesystem; tests/folders/build_output.rs covers it through clone_build_dirs.

use std::ffi::OsStr;
use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use super::Entry;

const NOT_MACOS: &str = "clonefile(2) is macOS only; nothing copied";

/// A directory by path; in Entry::File, another non-directory file by path.
pub(crate) struct DirFd(PathBuf);

impl DirFd {
    /// Why every clone is refused here, before anything is made.
    pub(crate) const NO_CLONE: Option<&'static str> = Some(NOT_MACOS);

    /// The folder at `path`, taken as it is: it is trusted, and nothing is opened.
    pub(crate) fn open(path: &Path) -> io::Result<DirFd> {
        Ok(DirFd(path.to_path_buf()))
    }

    /// What `name` is in this directory, looked at without following a symlink there.
    pub(crate) fn entry(&self, name: &OsStr) -> Entry {
        let path = self.0.join(name);
        match fs::symlink_metadata(&path) {
            Ok(meta) if meta.file_type().is_symlink() => Entry::Symlink,
            Ok(meta) if meta.is_dir() => Entry::Dir(DirFd(path)),
            Ok(_) => Entry::File(DirFd(path)),
            Err(e) if e.kind() == ErrorKind::NotFound => Entry::Missing,
            Err(e) => Entry::Unseen(e.to_string()),
        }
    }

    /// Makes nothing: refuses as NO_CLONE does.
    pub(crate) fn make_dir(&self, _name: &OsStr) -> Result<(), String> {
        Err(NOT_MACOS.to_string())
    }

    /// The same directory again.
    pub(crate) fn try_clone(&self) -> Result<DirFd, String> {
        Ok(DirFd(self.0.clone()))
    }

    /// The same directory again: by path there is no search-only open to make readable.
    pub(crate) fn readable(&self) -> Result<DirFd, String> {
        Ok(DirFd(self.0.clone()))
    }

    /// Clones nothing: refuses as NO_CLONE does.
    pub(crate) fn clone_into(&self, _target: &DirFd, _name: &OsStr) -> Result<(), String> {
        Err(NOT_MACOS.to_string())
    }
}
