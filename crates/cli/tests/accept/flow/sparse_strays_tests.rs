use super::sparse_strays;

fn files(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| name.to_string()).collect()
}

#[test]
fn a_folder_of_its_path_and_the_root_file_has_no_strays() {
    assert!(sparse_strays(&files(&["Cargo.toml", "src/a/a.txt", "src/a/deep/x.txt"]), &["src/a"], &["Cargo.toml"])
        .is_empty());
}

#[test]
fn another_zone_s_files_are_strays() {
    assert_eq!(
        sparse_strays(&files(&["Cargo.toml", "src/a/a.txt", "src/b/b.txt"]), &["src/a"], &["Cargo.toml"]),
        vec!["src/b/b.txt".to_string()]
    );
}

#[test]
fn a_sibling_folder_sharing_the_prefix_is_a_stray() {
    assert_eq!(sparse_strays(&files(&["src/ab/x.txt"]), &["src/a"], &["Cargo.toml"]), vec!["src/ab/x.txt".to_string()]);
}

#[test]
fn a_root_file_not_named_is_a_stray() {
    assert_eq!(sparse_strays(&files(&["README", "src/b/b.txt"]), &["src/b"], &["Cargo.toml"]), vec!["README".to_string()]);
}
