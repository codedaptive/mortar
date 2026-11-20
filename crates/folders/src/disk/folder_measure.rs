//! What measure_folder measured.
//! A bare type; the integration group covers it through measure_folder's cases.

/// A folder's `files` (everything but `.git` and the build dir) and `build` (the build dir alone), both apparent
/// sizes in bytes, and `newest`, the newest mtime over both in unix seconds (None: nothing found).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FolderMeasure {
    pub(crate) files: u64,
    pub(crate) build: u64,
    pub(crate) newest: Option<u64>,
}
