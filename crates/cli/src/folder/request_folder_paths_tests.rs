//! request_folder_paths: the work order's paths, then the root files, each path once at its first place.

use std::collections::BTreeMap;

use super::request_folder_paths;
use crate::config::Config;

fn config(root_files: &[&str]) -> Config {
    Config {
        origin_branch: "main".to_string(),
        integration_branch: "integration".to_string(),
        remote: "origin".to_string(),
        zones: BTreeMap::new(),
        root_files: root_files.iter().map(ToString::to_string).collect(),
        build_dirs: Vec::new(),
        cap_bytes: 1,
        floor_bytes: 0,
        formatter: None,
    }
}

fn strings(words: &[&str]) -> Vec<String> {
    words.iter().map(ToString::to_string).collect()
}

#[test]
fn the_paths_come_first_then_the_root_files() {
    assert_eq!(
        request_folder_paths(&strings(&["src/b", "src/a"]), &config(&["Cargo.toml", "Makefile"])),
        strings(&["src/b", "src/a", "Cargo.toml", "Makefile"])
    );
}

#[test]
fn a_root_file_already_given_as_a_path_is_not_repeated() {
    assert_eq!(
        request_folder_paths(&strings(&["Cargo.toml", "src/a"]), &config(&["Cargo.toml", "Makefile"])),
        strings(&["Cargo.toml", "src/a", "Makefile"])
    );
}

#[test]
fn a_path_given_twice_is_kept_once() {
    assert_eq!(request_folder_paths(&strings(&["src/a", "src/a"]), &config(&[])), strings(&["src/a"]));
}

#[test]
fn with_no_root_files_the_paths_are_all() {
    assert_eq!(request_folder_paths(&strings(&["src/a"]), &config(&[])), strings(&["src/a"]));
}
