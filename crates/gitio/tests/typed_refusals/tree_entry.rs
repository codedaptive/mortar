//! tree_entry_typed's answers and NotABlob refusals, and the untyped tree_entry's text for a folder and a gitlink.
use crate::fixture::{blob, commit, files, repo, tree};
use crate::support::GitRepo;
use gitio::{EntryKind, GitError, Oid, TreeEntryError};
use std::path::{Path, PathBuf};

/// A repository whose tree holds a file `f`, a folder `dir` (holding `x`) and a gitlink `sub`; and `f`'s blob id.
fn holding_file_folder_and_gitlink() -> (GitRepo, Oid, Oid) {
    let r = repo();
    let file = blob(&r.git, b"file\n");
    let inner = blob(&r.git, b"inner\n");
    let submodule = commit(&r.git, &files(&r.git, &[("x", b"submodule\n")]));
    let root = tree(&r.git, &[("100644", "f", &file), ("100644", "dir/x", &inner), ("160000", "sub", &submodule)]);
    (r, root, file)
}

fn refusal(stderr: &str) -> Result<Option<Oid>, GitError> {
    Err(GitError { args: Vec::new(), code: None, stderr: stderr.to_string() })
}

#[test]
fn a_file_gives_its_blob_id_and_a_missing_path_gives_none() {
    let (r, root, file) = holding_file_folder_and_gitlink();
    assert_eq!(r.git.tree_entry_typed(&root, Path::new("f")), Ok(Some(file)));
    assert_eq!(r.git.tree_entry_typed(&root, Path::new("nosuch")), Ok(None));
}

#[test]
fn a_folder_is_refused_as_not_a_blob_of_kind_tree() {
    let (r, root, _) = holding_file_folder_and_gitlink();
    let typed = r.git.tree_entry_typed(&root, Path::new("dir"));
    assert_eq!(typed, Err(TreeEntryError::NotABlob { path: PathBuf::from("dir"), kind: EntryKind::Tree }));
    assert_eq!(r.git.tree_entry(&root, Path::new("dir")), refusal("not a blob: \"dir\" is a \"tree\""));
}

#[test]
fn a_gitlink_is_refused_as_not_a_blob_of_kind_commit() {
    let (r, root, _) = holding_file_folder_and_gitlink();
    let typed = r.git.tree_entry_typed(&root, Path::new("sub"));
    assert_eq!(typed, Err(TreeEntryError::NotABlob { path: PathBuf::from("sub"), kind: EntryKind::Commit }));
    // join's classify_conflicted matches this text.
    assert_eq!(r.git.tree_entry(&root, Path::new("sub")), refusal("not a blob: \"sub\" is a \"commit\""));
}
