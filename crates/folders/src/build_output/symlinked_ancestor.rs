//! Finds a symlink among the folders a build dir sits in, below a trusted root.
//! No unit test: it reads the filesystem; tests/folders/build_output.rs covers it.

use std::fs;
use std::path::{Component, Path, PathBuf};

/// The first ancestor of `dir` under `root` that is a symlink, as a path relative to `root`; `dir` itself (its
/// last component) is not looked at, nor is `root` or anything above it: the caller's folder paths are trusted
/// (on macOS `/tmp` itself is a symlink). Answers None at the first ancestor that does not exist (create_dir_all
/// then makes real dirs below it) or cannot be read (nothing below it can be reached either), and at a component
/// that is not a plain name, since build_dir_outcome refuses such a dir.
pub(crate) fn symlinked_ancestor(root: &Path, dir: &str) -> Option<PathBuf> {
    let mut prefix = PathBuf::new();
    for component in Path::new(dir).parent()?.components() {
        let Component::Normal(name) = component else {
            return None;
        };
        prefix.push(name);
        match fs::symlink_metadata(root.join(&prefix)) {
            Ok(meta) if meta.file_type().is_symlink() => return Some(prefix),
            Ok(_) => {}
            Err(_) => return None,
        }
    }
    None
}
