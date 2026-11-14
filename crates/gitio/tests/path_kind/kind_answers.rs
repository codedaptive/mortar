//! path_kind's answer for each kind of stored entry, for a missing path and a path under a file, by tree id and by
//! commit id.
use crate::fixture::holding_each_kind;
use gitio::PathKind::{File, Folder, Gitlink};
use std::path::Path;

#[test]
fn kind_a_file_and_a_symlink_are_files() {
    let (r, root, _) = holding_each_kind();
    assert_eq!(r.git.path_kind(&root, Path::new("file")), Ok(Some(File)));
    assert_eq!(r.git.path_kind(&root, Path::new("link")), Ok(Some(File)));
}

#[test]
fn kind_names_starting_with_a_colon_or_holding_a_newline_are_files() {
    let (r, root, _) = holding_each_kind();
    assert_eq!(r.git.path_kind(&root, Path::new(":colon")), Ok(Some(File)));
    assert_eq!(r.git.path_kind(&root, Path::new("new\nline")), Ok(Some(File)));
}

#[test]
fn kind_a_folder_is_a_folder() {
    let (r, root, _) = holding_each_kind();
    assert_eq!(r.git.path_kind(&root, Path::new("dir")), Ok(Some(Folder)));
}

#[test]
fn kind_a_gitlink_is_a_gitlink() {
    let (r, root, _) = holding_each_kind();
    assert_eq!(r.git.path_kind(&root, Path::new("sub")), Ok(Some(Gitlink)));
}

#[test]
fn kind_a_missing_path_and_a_path_under_a_file_give_none() {
    let (r, root, _) = holding_each_kind();
    assert_eq!(r.git.path_kind(&root, Path::new("missing")), Ok(None));
    assert_eq!(r.git.path_kind(&root, Path::new("file/x")), Ok(None));
}

#[test]
fn kind_a_commit_id_answers_as_its_tree() {
    let (r, _, head) = holding_each_kind();
    assert_eq!(r.git.path_kind(&head, Path::new("file")), Ok(Some(File)));
    assert_eq!(r.git.path_kind(&head, Path::new("dir")), Ok(Some(Folder)));
    assert_eq!(r.git.path_kind(&head, Path::new("sub")), Ok(Some(Gitlink)));
    assert_eq!(r.git.path_kind(&head, Path::new("missing")), Ok(None));
}
