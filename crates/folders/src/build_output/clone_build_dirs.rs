//! Shares a parent folder's build dirs with a newly made folder.
//! No unit test: it reads and writes the filesystem; tests/folders/build_output.rs covers it.

use std::ffi::OsStr;
use std::fs;
use std::io::{self, ErrorKind};
use std::path::Path;

use super::{
    ancestor_link_why, ancestor_refusal, build_dir_outcome, walk_ancestors, AncestorWalk, BuildDirClone, DirFd, Entry,
};

/// Clones each of `build_dirs` from `parent` into `folder` with clonefile(2), in order, and answers one
/// outcome per dir. `parent` is the folder whose build output is shared (for a request folder, its zone
/// folder); `folder` is the newly made one: call this after make_request_folder or make_zone_folder returns.
/// `build_dirs` are the caller's, relative to both folders (a caller with no list passes DEFAULT_BUILD_DIR).
/// On macOS only: the two folders, which are trusted, are opened once by path, and a missing `folder` is made by path
/// when a dir is cloned into it; below them every ancestor of a dir, on both sides, is opened by descriptor,
/// search-only and without following a symlink; a missing folder ancestor is made with mkdirat, only when the dir is
/// cloned, and opened the same way. A dir with a symlinked ancestor in either folder is Refused naming it, with nothing
/// cloned or created past it. A symlink at the dir itself in the parent is Refused; otherwise, on macOS, the clone is
/// made with fclonefileat from the file that look opened (a directory is reopened for reading through its own
/// descriptor) into the folder ancestor's descriptor, so nothing put in place by name afterwards is cloned. A dir
/// already in `folder` is Kept, never overwritten or merged, so a repeat make is safe. A refused dir has nothing
/// copied: there is never a byte copy. Off macOS nothing is opened or made: the looks are by path, the outcomes other
/// than a clone are the same, and every clone is Refused.
pub fn clone_build_dirs(parent: &Path, folder: &Path, build_dirs: &[String]) -> Vec<BuildDirClone> {
    let parent_root = DirFd::open(parent);
    let folder_root = DirFd::open(folder);
    build_dirs.iter().map(|dir| clone_build_dir(&parent_root, &folder_root, folder, dir)).collect()
}

/// One dir's outcome: a symlinked ancestor on either side first, then build_dir_outcome on what each side holds. A
/// root that could not be opened holds nothing; a name that cannot be looked up is not there.
fn clone_build_dir(
    parent_root: &io::Result<DirFd>,
    folder_root: &io::Result<DirFd>,
    folder: &Path,
    dir: &str,
) -> BuildDirClone {
    let parent_walk = look(parent_root, dir);
    let folder_probe = look(folder_root, dir);
    if let Some(refused) = ancestor_refusal(dir, link_of(&parent_walk), link_of(&folder_probe)) {
        return refused;
    }
    // A dir with no last name is not a plain path, so build_dir_outcome refuses it whatever the facts say.
    let name = Path::new(dir).file_name();
    let found = match (parent_walk, name) {
        (AncestorWalk::Reached(holder), Some(name)) => Some((holder.entry(name), name)),
        _ => None,
    };
    let in_folder = match (&folder_probe, name) {
        (AncestorWalk::Reached(holder), Some(name)) => {
            !matches!(holder.entry(name), Entry::Missing | Entry::Unseen(_))
        }
        _ => false,
    };
    let (parent_is_symlink, source) = parent_facts(found);
    let clone = source.map(|source| {
        move || {
            if let Some(why) = DirFd::NO_CLONE {
                return Err(why.to_string());
            }
            let target = clone_target(folder_root, folder, dir)?;
            let (source, name) = source?;
            source.clone_into(&target, name)
        }
    });
    build_dir_outcome(dir, parent_is_symlink, in_folder, clone)
}

/// What a clone is made from, with its name, or why it is refused.
type CloneSource<'a> = Result<(DirFd, &'a OsStr), String>;

/// What the parent holds at `dir`'s last name: whether it is a symlink, and what a clone would be made from (the file
/// the look opened, or the directory it opened reopened for reading, with its name); None when there is nothing to
/// clone from (nothing there, or a symlink). A directory or file that is there but cannot be opened has a source whose
/// clone is refused with the open's error, as clonefile met it.
fn parent_facts(found: Option<(Entry, &OsStr)>) -> (bool, Option<CloneSource<'_>>) {
    let refused = |why: String| format!("clonefile refused: {why}");
    match found {
        Some((Entry::Dir(searched), name)) => {
            (false, Some(searched.readable().map(|source| (source, name)).map_err(refused)))
        }
        Some((Entry::File(source), name)) => (false, Some(Ok((source, name)))),
        Some((Entry::Other(why) | Entry::Unreadable(why), _)) => (false, Some(Err(refused(why)))),
        Some((Entry::Symlink, _)) => (true, None),
        Some((Entry::Missing | Entry::Unseen(_), _)) | None => (false, None),
    }
}

/// The folder directory a clone of `dir` goes into: its ancestors walked again from the folder's root, missing ones
/// made, each opened without following a symlink. A root that is missing is first made by path with create_dir_all
/// (it is trusted, as its open is); one that cannot be opened is reported against itself. A holder (the dir's last
/// ancestor) that is a directory there but cannot be opened refuses the clone as clonefile would ("clonefile refused:
/// <os error>"); a symlinked ancestor is refused in ancestor_link_why's words; any other make or walk that fails, an
/// unopenable ancestor further up or a holder that is a file included, is reported against the dir's holder in the
/// folder, whatever ancestor failed.
fn clone_target(folder_root: &io::Result<DirFd>, folder: &Path, dir: &str) -> Result<DirFd, String> {
    let target = folder.join(dir);
    let holder = target.parent().unwrap_or(folder);
    let could_not_make = |why: String| format!("could not make {}: {why}; nothing copied", holder.display());
    let made;
    let root = match folder_root {
        Ok(root) => root,
        Err(e) if e.kind() == ErrorKind::NotFound => {
            made = fs::create_dir_all(folder)
                .and_then(|()| DirFd::open(folder))
                .map_err(|e| could_not_make(e.to_string()))?;
            &made
        }
        Err(e) => return Err(format!("could not open {}: {e}; nothing copied", folder.display())),
    };
    match walk_ancestors(root, dir, true) {
        AncestorWalk::Reached(holder) => Ok(holder),
        AncestorWalk::Symlink(link) => Err(ancestor_link_why(dir, "folder's", &link)),
        AncestorWalk::Missing => Err(could_not_make("a component is not a plain name".to_string())),
        AncestorWalk::Failed(why) => Err(could_not_make(why)),
        AncestorWalk::Unopened(why) => Err(format!("clonefile refused: {why}")),
    }
}

/// The walk on one side, or Missing when that side's root could not be opened.
fn look(root: &io::Result<DirFd>, dir: &str) -> AncestorWalk {
    root.as_ref().map_or(AncestorWalk::Missing, |root| walk_ancestors(root, dir, false))
}

/// The symlinked ancestor a walk stopped at, if it stopped at one.
fn link_of(walk: &AncestorWalk) -> Option<&Path> {
    match walk {
        AncestorWalk::Symlink(link) => Some(link),
        _ => None,
    }
}
