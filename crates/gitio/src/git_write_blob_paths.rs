//! Git::write_blob_paths: write a tree's version of named paths into a folder. It only runs git, so it has no unit
//! tests; Nagatha's integration test "write_blob_paths writing a tree's version of named paths, a glob-like name taken
//! literally, and writing nothing when one path is missing from the tree" is to cover it.

use crate::write_blob_paths_request::write_blob_paths_request;
use crate::{Git, GitError, Oid};
use std::path::{Path, PathBuf};

impl Git {
    /// Writes `tree`'s version of each of `paths` into the files and the index of `folder`, over local edits; a folder
    /// path writes everything under it. A path `tree` lacks is git's error (exit 1), and then nothing is written.
    pub fn write_blob_paths(&self, folder: &Path, tree: &Oid, paths: &[PathBuf]) -> Result<(), GitError> {
        let (args, opts) = write_blob_paths_request(folder, tree, paths)?;
        self.run(&args, &opts).map(drop)
    }
}
