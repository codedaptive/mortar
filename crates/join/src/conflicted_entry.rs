//! ConflictedEntry: what a resolve's first run does with one conflicted path. A declaration only;
//! classify_conflicted_tests exercises it.

/// What the conflicted tree holds at a conflicted path, chosen by classify_conflicted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ConflictedEntry {
    /// A file: its conflicted version is written into the folder.
    Write,
    /// The conflicted tree has no entry there (a rename/rename conflict): skipped and listed.
    Absent,
    /// A submodule (gitlink): it has no file text, so nothing is written and it is listed apart.
    Gitlink,
}
