//! Git::tree_with_paths_typed: a tree made of a base tree with some paths' entries taken from another tree, each
//! refusal typed. It only runs git, so it has no unit tests (its refusal checks are tested where they live:
//! tree_with_paths_list_request and read_path_entries); the zone's integration tests (tests/typed/tree_with_paths.rs,
//! tests/typed_refusals/tree_with_paths.rs) cover it: "tree_with_paths keeping base outside
//! paths; a path changed, added and removed; a symlink and a gitlink taken; a path absent from both trees; a folder in from refused; a folder in base that from lacks refused; a folder and a path under it both
//! named refused; a file replaced by a folder's entry and back; a ':'-leading and newline-holding name taken
//! literally; a decomposed-Unicode name taken on macOS; empty paths giving base's tree; the folder's index and files
//! untouched and no temp index left behind; a rename/rename conflicted path absent from merge_tree's tree; each
//! refusal arriving as its TreeWithPathsRefusal variant carrying its path(s); a bad base arriving as Git".

use crate::absent_paths::absent_paths;
use crate::index_info_stdin::index_info_stdin;
use crate::next_temp_index::next_temp_index;
use crate::oid_line::oid_line;
use crate::read_path_entries::read_path_entries;
use crate::tree_with_paths_list_request::tree_with_paths_list_request;
use crate::tree_with_paths_requests::tree_with_paths_requests;
use crate::{Git, Oid, TreeWithPathsError, TreeWithPathsRefusal};
use std::path::PathBuf;

impl Git {
    /// `base`'s tree with each of `paths` taking its entry (mode and object) from `from`, and a path `from` lacks
    /// removed. Built in a temporary index, so the folder's index and files are untouched; the index is removed on
    /// every return path. Writes objects; moves no ref. Empty `paths` answers base's tree (git still runs read-tree,
    /// update-index with empty input and write-tree, so a bad base is Git). Paths name file entries; each refusal is
    /// Refused, and every refusal comes before the temporary index is made:
    /// - Unusable: an empty path or one holding a NUL;
    /// - NotAsStored: a path with a leading or trailing '/', or an empty, '.' or '..' part;
    /// - Overlap: two paths where one is a folder prefix of the other, naming both;
    /// - FolderInFrom: a path naming a folder in `from`;
    /// - FolderInBase: a path `from` lacks that names a folder in `base` (removal drops only an entry of exactly that
    ///   name, so the folder would stay).
    ///
    /// A git run that fails, an ls-tree answer of the wrong shape, or a write-tree answer that is not one id line, is
    /// Git.
    pub fn tree_with_paths_typed(&self, base: &Oid, from: &Oid, paths: &[PathBuf]) -> Result<Oid, TreeWithPathsError> {
        let entries = match tree_with_paths_list_request(from, paths)? {
            Some((args, opts)) => {
                let listed = self.run(&args, &opts).map_err(TreeWithPathsError::Git)?;
                read_path_entries(&listed.stdout, paths, |path| TreeWithPathsRefusal::FolderInFrom { path })?
            }
            None => Vec::new(),
        };
        let absent = absent_paths(paths, &entries);
        // Only for its refusal of a folder in base: the entries base holds for absent paths are removed anyway.
        if let Some((args, opts)) = tree_with_paths_list_request(base, &absent)? {
            let listed = self.run(&args, &opts).map_err(TreeWithPathsError::Git)?;
            read_path_entries(&listed.stdout, &absent, |path| TreeWithPathsRefusal::FolderInBase { path })?;
        }
        let stdin = index_info_stdin(paths, &entries, base.as_str().len());
        let index = next_temp_index();
        let [read_tree, update, write_tree] = tree_with_paths_requests(base, &index.0, stdin);
        self.run(&read_tree.0, &read_tree.1).map_err(TreeWithPathsError::Git)?;
        self.run(&update.0, &update.1).map_err(TreeWithPathsError::Git)?;
        oid_line(&self.run(&write_tree.0, &write_tree.1).map_err(TreeWithPathsError::Git)?.stdout)
            .map_err(TreeWithPathsError::Git)
    }
}
