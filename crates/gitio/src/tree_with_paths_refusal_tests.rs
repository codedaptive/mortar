use crate::TreeWithPathsRefusal;
use std::path::PathBuf;

#[test]
fn a_folder_in_from_and_a_folder_in_base_on_the_same_path_differ() {
    assert_ne!(
        TreeWithPathsRefusal::FolderInFrom { path: PathBuf::from("dir") },
        TreeWithPathsRefusal::FolderInBase { path: PathBuf::from("dir") }
    );
}

#[test]
fn the_same_variant_on_the_same_path_is_equal() {
    assert_eq!(
        TreeWithPathsRefusal::NotAsStored { path: PathBuf::from("a//b") },
        TreeWithPathsRefusal::NotAsStored { path: PathBuf::from("a//b") }
    );
}

#[test]
fn the_same_variant_on_another_path_differs() {
    assert_ne!(
        TreeWithPathsRefusal::Unusable { path: PathBuf::from("") },
        TreeWithPathsRefusal::Unusable { path: PathBuf::from("x") }
    );
}

#[test]
fn an_overlap_with_folder_and_inner_swapped_differs() {
    assert_ne!(
        TreeWithPathsRefusal::Overlap { folder: PathBuf::from("f"), inner: PathBuf::from("f/x") },
        TreeWithPathsRefusal::Overlap { folder: PathBuf::from("f/x"), inner: PathBuf::from("f") }
    );
}

#[test]
fn a_clone_is_equal_to_its_original() {
    let overlap = TreeWithPathsRefusal::Overlap { folder: PathBuf::from("f"), inner: PathBuf::from("f/x") };
    assert_eq!(
        overlap.clone(),
        TreeWithPathsRefusal::Overlap { folder: PathBuf::from("f"), inner: PathBuf::from("f/x") }
    );
}
