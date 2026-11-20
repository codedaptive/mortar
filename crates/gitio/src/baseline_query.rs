//! BaselineQuery: which git read Git::output_chars counts. A declaration only; output_chars_request_tests exercises it.

use std::path::PathBuf;

/// One git read whose output is counted. In every variant `paths` are pathspecs sent after `--`, matched as git reads
/// them (globs included), relative to the folder the query runs in; none means the whole tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BaselineQuery {
    /// `git log -p <range> -- <paths>`: the commits of a revision range, each with its patch.
    LogPatches {
        /// A revision range as git reads it, such as `<a>..<b>`.
        range: String,
        /// Pathspecs limiting the commits and patches shown.
        paths: Vec<PathBuf>,
    },
    /// `git diff <a> <b> -- <paths>`: the patch between two revisions.
    Diff {
        /// The revision diffed from.
        a: String,
        /// The revision diffed to.
        b: String,
        /// Pathspecs limiting the patch.
        paths: Vec<PathBuf>,
    },
    /// `git diff HEAD -- <paths>`: the patch from HEAD to the folder's files, staged and unstaged.
    DiffHead {
        /// Pathspecs limiting the patch.
        paths: Vec<PathBuf>,
    },
    /// `git status --long -- <paths>`: the folder's status as a person reads it.
    Status {
        /// Pathspecs limiting the paths reported.
        paths: Vec<PathBuf>,
    },
}
