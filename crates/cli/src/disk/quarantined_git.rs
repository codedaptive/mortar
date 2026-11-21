//! quarantined_git: the git that with_object_quarantine hands its closure, built without touching the file system.

use std::ffi::OsString;
use std::path::Path;

use gitio::Git;

/// A clone of `git` whose env keeps the caller's pairs in order, then adds GIT_OBJECT_DIRECTORY=`dir`, where git
/// writes new objects, and GIT_ALTERNATE_OBJECT_DIRECTORIES=`objects`, where it still reads the repository's own.
pub(crate) fn quarantined_git(git: &Git, dir: &Path, objects: &Path) -> Git {
    let mut quarantined = git.clone();
    quarantined.env.push((OsString::from("GIT_OBJECT_DIRECTORY"), dir.as_os_str().to_owned()));
    quarantined.env.push((OsString::from("GIT_ALTERNATE_OBJECT_DIRECTORIES"), objects.as_os_str().to_owned()));
    quarantined
}
