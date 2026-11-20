//! The cone a folder hands git's cone-mode commands: its top-level files dropped, by what its tree holds.
//! No unit test: it runs git; cone_folders_tests cover the decision; tests/folders/root_files.rs covers it in
//! a throwaway repository.

use std::path::Path;

use gitio::{Git, Oid};

use super::FolderError;
use crate::plan::cone_folders;

/// cone_folders over `cone`, each top-level path's kind read from the commit `folder`'s HEAD points at, or from
/// `fallback` (a tree or commit) when HEAD is unborn. Every git error is Git.
pub(crate) fn git_cone(git: &Git, folder: &Path, fallback: &Oid, cone: &[String]) -> Result<Vec<String>, FolderError> {
    let tree = git.head(folder).map_err(FolderError::Git)?.1.unwrap_or_else(|| fallback.clone());
    cone_folders(cone, |path| git.path_kind(&tree, Path::new(path))).map_err(FolderError::Git)
}
