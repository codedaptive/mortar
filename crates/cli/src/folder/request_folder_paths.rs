//! The sparse paths of a request folder: the work order's paths, then the config's root files.

use crate::config::Config;

/// `paths` in order, then `config.root_files` in order; a path given more than once is kept at its first place only.
pub fn request_folder_paths(paths: &[String], config: &Config) -> Vec<String> {
    let mut all: Vec<String> = Vec::new();
    for path in paths.iter().chain(&config.root_files) {
        if !all.contains(path) {
            all.push(path.clone());
        }
    }
    all
}
