//! One folder's size and last touch, the one measure both disk and sweep_build judge idleness by.
//! No unit test: it reads the file system. The integration group is to cover, in a throwaway
//! directory: files leaves out `.git` and the build dir, build is the build dir alone, newest is the later of the
//! two; a folder with no build dir has build 0 and newest from its files.

use std::path::Path;

use super::{measure_tree, FolderMeasure};
use crate::make::FolderError;
use crate::sweep::DEFAULT_BUILD_DIR;

/// files: measure_tree of `path` skipping `.git` and DEFAULT_BUILD_DIR; build: measure_tree of
/// `path`/DEFAULT_BUILD_DIR; newest: the later of their newest times.
pub(crate) fn measure_folder(path: &Path) -> Result<FolderMeasure, FolderError> {
    let files = measure_tree(path, &[".git", DEFAULT_BUILD_DIR])?;
    let build = measure_tree(&path.join(DEFAULT_BUILD_DIR), &[])?;
    Ok(FolderMeasure {
        files: files.bytes,
        build: build.bytes,
        newest: files.newest.max(build.newest),
    })
}
