//! What one name in a directory is, as DirFd::entry finds it without following a symlink.
//! A bare type; tests/folders/build_output.rs covers it through clone_build_dirs.

use super::DirFd;

/// One name inside a DirFd. Dir and File hold what they found as a DirFd: open by descriptor on macOS, by path
/// elsewhere.
pub(crate) enum Entry {
    /// A real directory (on macOS, open search-only).
    Dir(DirFd),
    /// Any other file that is not a symlink: a regular file, a fifo.
    File(DirFd),
    /// A symlink: neither followed nor opened.
    Symlink,
    /// Nothing by that name.
    Missing,
    /// A directory that is there but could not be opened (macOS only: elsewhere nothing is opened); the text is the
    /// OS error of the open.
    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
    Other(String),
    /// A file that is neither a directory nor a symlink and could not be opened (macOS only, as Other); the text is
    /// the OS error of the open.
    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
    Unreadable(String),
    /// A name whose lookup itself failed (faccessat on macOS, lstat elsewhere), so whether it is there is unknown;
    /// the text is that lookup's OS error.
    Unseen(String),
}
