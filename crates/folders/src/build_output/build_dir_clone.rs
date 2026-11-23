//! What clone_build_dirs did with one build dir.
//! A bare type; build_dir_outcome_tests.rs covers each case and tests/folders/build_output.rs covers it through
//! clone_build_dirs.

/// One build dir's outcome, named by `dir` as the caller gave it (relative to both folders).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildDirClone {
    /// Cloned from the parent with clonefile(2): it shares the parent's blocks until a file changes.
    Shared { dir: String },
    /// The parent has no such dir: nothing to do, not an error.
    Absent { dir: String },
    /// The folder already has the dir (a repeat make): left as it is, nothing cloned.
    Kept { dir: String },
    /// Not shared and nothing copied; `why` says why.
    Refused { dir: String, why: String },
}
