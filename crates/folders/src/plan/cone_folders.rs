//! The paths of a cone that git's cone-mode commands accept: top-level files dropped (S-126).

use gitio::PathKind;

/// `cone` without its top-level files, in its own order. Git's cone-mode sparse-checkout set and add refuse a
/// top-level file once the index is filled ("'<path>' is not a directory"), and cone mode always checks out the
/// top-level files, so dropping them changes nothing in the checkout (S-126).
///
/// A path holding a '/' is kept without asking `kind_of`. For a top-level path `kind_of` is asked once, and the
/// path is dropped only when it answers File; Folder, Gitlink and None (absent from the tree, which git's cone
/// commands accept) are kept. The result may be empty. The first error from `kind_of` is returned at once.
pub(crate) fn cone_folders<E>(
    cone: &[String],
    mut kind_of: impl FnMut(&str) -> Result<Option<PathKind>, E>,
) -> Result<Vec<String>, E> {
    let mut folders = Vec::new();
    for path in cone {
        if path.contains('/') || kind_of(path)? != Some(PathKind::File) {
            folders.push(path.clone());
        }
    }
    Ok(folders)
}
