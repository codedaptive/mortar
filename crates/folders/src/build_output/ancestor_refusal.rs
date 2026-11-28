//! Turns a symlinked ancestor of a build dir, in the parent or the folder, into a refusal.

use std::path::Path;

use super::{ancestor_link_why, BuildDirClone};

/// Refused when either side has a symlinked ancestor of `dir`, naming it (the parent's is named when both do);
/// None when neither does. A symlinked ancestor would let the clone read from or write to a place outside the
/// two folders, so nothing is copied.
pub(crate) fn ancestor_refusal(
    dir: &str,
    parent_link: Option<&Path>,
    folder_link: Option<&Path>,
) -> Option<BuildDirClone> {
    let (side, link) = parent_link
        .map(|link| ("parent's", link))
        .or(folder_link.map(|link| ("folder's", link)))?;
    Some(BuildDirClone::Refused { dir: dir.to_string(), why: ancestor_link_why(dir, side, link) })
}
