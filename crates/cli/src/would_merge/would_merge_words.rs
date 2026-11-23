//! WouldMergeWords: the words of `mr would-merge` read into the squashes it asks about.
//! No sibling test: a bare type with no behaviour of its own; parse_would_merge_args_tests builds it.

use crate::landing::SquashName;

/// The squash to try into integration, then the further squashes (each `--with`, in the order given) to try onto
/// its result; `with` is empty for the plain one-squash form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WouldMergeWords {
    pub squash: SquashName,
    pub with: Vec<SquashName>,
}
