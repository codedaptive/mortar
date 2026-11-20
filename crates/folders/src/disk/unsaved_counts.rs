//! A folder's unsaved work, as counts.
//! A bare type; count_unsaved_tests exercises it.

/// `tracked`: the paths changed against HEAD in the index or the files, plus the conflicted paths. `untracked`: the
/// files git does not track, each counted. Ignored files are in neither.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UnsavedCounts {
    pub tracked: u64,
    pub untracked: u64,
}
