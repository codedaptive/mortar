//! What measure_tree measured.
//! A bare type; Nagatha's integration group covers it through measure_tree's cases.

/// `bytes`, the apparent size of a tree's files; `newest`, its newest mtime in unix seconds (None: nothing found).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TreeSize {
    pub(crate) bytes: u64,
    pub(crate) newest: Option<u64>,
}
