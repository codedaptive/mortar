//! Git::tree_with_paths: a tree made of a base tree with some paths' entries taken from another tree. It only runs
//! git, so it has no unit tests; Nagatha's integration tests are to cover it: "tree_with_paths keeping base outside
//! paths; a path changed, added and removed; a symlink and a gitlink taken; a path absent from both trees; a folder in
//! from refused; a folder in base that from lacks refused; a folder and a path under it both named refused; a file
//! replaced by a folder's entry and back; a ':'-leading and newline-holding name taken literally; a decomposed-Unicode
//! name taken on macOS; empty paths giving base's tree; the folder's index and files untouched and no temp index left
//! behind; a rename/rename conflicted path absent from merge_tree's tree".

use crate::absent_paths::absent_paths;
use crate::index_info_stdin::index_info_stdin;
use crate::next_temp_index::next_temp_index;
use crate::oid_line::oid_line;
use crate::read_path_entries::read_path_entries;
use crate::tree_with_paths_list_request::tree_with_paths_list_request;
use crate::tree_with_paths_requests::tree_with_paths_requests;
use crate::{Git, GitError, Oid};
use std::path::PathBuf;

impl Git {
    /// `base`'s tree with each of `paths` taking its entry (mode and object) from `from`, and a path `from` lacks
    /// removed. Built in a temporary index, so the folder's index and files are untouched; the index is removed on
    /// every return path. Writes objects; moves no ref. Empty `paths` answers base's tree (git still runs read-tree,
    /// update-index with empty input and write-tree, so a bad base is git's error). Paths name file entries: a path
    /// naming a folder in `from` is refused, and so is a path `from` lacks that names a folder in `base` (removal drops
    /// only an entry of exactly that name, so the folder would stay). A path that is a folder prefix of another, or
    /// not written as git stores it, is refused (tree_with_paths_list_request), as is any other path it refuses;
    /// every refusal comes before the temporary index is made.
    pub fn tree_with_paths(&self, base: &Oid, from: &Oid, paths: &[PathBuf]) -> Result<Oid, GitError> {
        let entries = match tree_with_paths_list_request(from, paths)? {
            Some((args, opts)) => read_path_entries(&self.run(&args, &opts)?.stdout, paths)?,
            None => Vec::new(),
        };
        let absent = absent_paths(paths, &entries);
        // Only for its refusal of a folder in base: the entries base holds for absent paths are removed anyway.
        if let Some((args, opts)) = tree_with_paths_list_request(base, &absent)? {
            read_path_entries(&self.run(&args, &opts)?.stdout, &absent)?;
        }
        let stdin = index_info_stdin(paths, &entries, base.as_str().len());
        let index = next_temp_index();
        let [read_tree, update, write_tree] = tree_with_paths_requests(base, &index.0, stdin);
        self.run(&read_tree.0, &read_tree.1)?;
        self.run(&update.0, &update.1)?;
        oid_line(&self.run(&write_tree.0, &write_tree.1)?.stdout)
    }
}
