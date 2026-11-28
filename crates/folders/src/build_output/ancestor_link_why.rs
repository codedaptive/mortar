//! The words a refusal gives for a build dir's ancestor that is a symlink, on either side.

use std::path::Path;

/// Why a build dir `dir` is refused when `link`, one of its ancestors on `side` ("parent's" or "folder's"), is a
/// symlink: names the link and the dir, and says nothing was copied. A symlinked ancestor would let the clone read
/// from or write to a place outside the two folders, so the clone stops there.
pub(crate) fn ancestor_link_why(dir: &str, side: &str, link: &Path) -> String {
    format!("the {side} \"{}\", an ancestor of build dir \"{dir}\", is a symlink; nothing copied", link.display())
}
