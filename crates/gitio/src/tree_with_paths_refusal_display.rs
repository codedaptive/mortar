//! Display for TreeWithPathsRefusal: the refusal's reason, naming each path in its `{:?}` form.

use crate::TreeWithPathsRefusal;
use std::fmt;
use std::os::unix::ffi::OsStrExt;

impl fmt::Display for TreeWithPathsRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TreeWithPathsRefusal::FolderInFrom { path } | TreeWithPathsRefusal::FolderInBase { path } => {
                write!(f, "not a file entry: {path:?} is a folder")
            }
            TreeWithPathsRefusal::Overlap { folder, inner } => {
                write!(f, "not usable paths (one is a folder holding the other): {folder:?} and {inner:?}")
            }
            TreeWithPathsRefusal::NotAsStored { path } if path.as_os_str().as_bytes().ends_with(b"/") => {
                write!(f, "not a usable path (ends in '/', naming a folder's children): {path:?}")
            }
            TreeWithPathsRefusal::NotAsStored { path } => write!(
                f,
                "not a usable path (not as git stores it: a leading '/', or an empty, '.' or '..' part): {path:?}"
            ),
            TreeWithPathsRefusal::Unusable { path } if path.as_os_str().is_empty() => {
                write!(f, "not a usable path (empty): {path:?}")
            }
            // The text no_nul gives for a path, so it matches usable_path's refusal.
            TreeWithPathsRefusal::Unusable { path } => {
                write!(f, "path holds a NUL: {:?}", String::from_utf8_lossy(path.as_os_str().as_bytes()))
            }
        }
    }
}
