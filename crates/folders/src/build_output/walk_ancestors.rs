//! Walks down a build dir's ancestors from a root (by descriptor on macOS, by path elsewhere), never following a
//! symlink.
//! No unit test: it opens and makes directories; tests/folders/build_output.rs covers it through clone_build_dirs.

use std::ffi::OsStr;
use std::io;
use std::path::{Component, Path, PathBuf};

use super::{AncestorWalk, DirFd, Entry};

/// Opens each ancestor of `dir` (every component but the last) inside the one before it, starting at `root`, each
/// without following a symlink, and answers where the walk ended. `root` itself is trusted and `dir`'s last component
/// is not looked at. With `make_missing` a missing ancestor is made (mkdirat on macOS) and then opened the same way,
/// so a symlink put there in between is still met as one, and an ancestor that cannot be made or walked into is
/// Failed (Unopened when it is the holder, a directory there but unopenable); without it nothing is made.
pub(crate) fn walk_ancestors(root: &DirFd, dir: &str, make_missing: bool) -> AncestorWalk {
    let mut at = match root.try_clone() {
        Ok(fd) => fd,
        Err(why) => return AncestorWalk::Failed(why),
    };
    let holder = Path::new(dir).parent();
    let mut prefix = PathBuf::new();
    for component in holder.into_iter().flat_map(Path::components) {
        let Component::Normal(name) = component else {
            return AncestorWalk::Missing;
        };
        prefix.push(name);
        let is_holder = holder == Some(prefix.as_path());
        at = match at.entry(name) {
            Entry::Dir(fd) => fd,
            Entry::Symlink => return AncestorWalk::Symlink(prefix),
            Entry::Missing if make_missing => match make_and_open(&at, name, &prefix, is_holder) {
                Ok(fd) => fd,
                Err(end) => return end,
            },
            found if make_missing => return in_the_way(found, is_holder),
            _ => return AncestorWalk::Missing,
        };
    }
    AncestorWalk::Reached(at)
}

/// Makes `name` in `at` and opens it without following a symlink; `prefix` is its path from the root.
fn make_and_open(at: &DirFd, name: &OsStr, prefix: &Path, is_holder: bool) -> Result<DirFd, AncestorWalk> {
    at.make_dir(name).map_err(AncestorWalk::Failed)?;
    match at.entry(name) {
        Entry::Dir(fd) => Ok(fd),
        Entry::Symlink => Err(AncestorWalk::Symlink(prefix.to_path_buf())),
        Entry::Missing => Err(AncestorWalk::Failed(format!("\"{}\" was gone again once made", prefix.display()))),
        found => Err(in_the_way(found, is_holder)),
    }
}

/// Where the walk ends at an ancestor it found as `found` (File, Unreadable, Other or Unseen), as create_dir_all of
/// the holder met it. Other, a directory that cannot be opened, is Unopened with the OS error of the open when it is
/// the holder itself (create_dir_all found the holder and the clone then failed), and Failed with that error when it
/// is further up (making the holder failed). Unseen is Failed with the error of the lookup. A file, readable or not
/// (File or Unreadable), is Failed with the error making a directory over it gives: EEXIST (17) when that file is the
/// holder itself, ENOTDIR (20) when it is further up; both numbers are the same on macOS and Linux.
fn in_the_way(found: Entry, is_holder: bool) -> AncestorWalk {
    match found {
        Entry::Other(why) if is_holder => AncestorWalk::Unopened(why),
        Entry::Other(why) | Entry::Unseen(why) => AncestorWalk::Failed(why),
        _ => AncestorWalk::Failed(io::Error::from_raw_os_error(if is_holder { 17 } else { 20 }).to_string()),
    }
}
