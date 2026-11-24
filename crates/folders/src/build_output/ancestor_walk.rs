//! Where walk_ancestors ended on its way down a build dir's ancestors.
//! A bare type; tests/folders/build_output.rs covers it through clone_build_dirs.

use std::path::PathBuf;

use super::DirFd;

/// The end of one walk_ancestors.
pub(crate) enum AncestorWalk {
    /// Every ancestor is a real directory: this is the one the build dir's last component sits in.
    Reached(DirFd),
    /// The first ancestor that is a symlink, relative to the root; nothing past it was opened or made.
    Symlink(PathBuf),
    /// Looking only: an ancestor does not exist, is not a directory, or could not be opened or looked up; or, either
    /// way, a component is not a plain name.
    Missing,
    /// Making: an ancestor is a file, could not be looked up, or (further up than the holder) could not be opened, or
    /// making one failed; or, looking too, the root could not be duplicated. The text is the reason alone.
    Failed(String),
    /// Making: the holder (the build dir's last ancestor) is a directory there but could not be opened (macOS only:
    /// elsewhere nothing is opened); the text is the OS error of the open.
    Unopened(String),
}
