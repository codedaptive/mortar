//! Decides one build dir's outcome from what is on disk, cloning only when nothing stands in the way.

use std::path::{Component, Path};

use super::BuildDirClone;

/// The outcome for `dir`; the first match wins:
/// a `dir` that is not a plain relative path inside the folder (empty, absolute, `.` or a `./` prefix, or holding `..`) is
/// Refused; a parent dir that is a symlink is Refused, since a clone would point back into the parent; a dir with no
/// `clone` is otherwise Absent (`clone` is None when the parent has nothing to clone from at `dir`: nothing there, or
/// a symlink, which the check before has already refused); a dir the folder already has (`in_folder`) is Kept;
/// otherwise `clone` runs, and its Ok is Shared and its Err is Refused with that reason. `clone` runs only in that
/// last case.
pub(crate) fn build_dir_outcome(
    dir: &str,
    parent_is_symlink: bool,
    in_folder: bool,
    clone: Option<impl FnOnce() -> Result<(), String>>,
) -> BuildDirClone {
    let dir = dir.to_string();
    let path = Path::new(&dir);
    let plain = path.components().next().is_some()
        && path.components().all(|c| matches!(c, Component::Normal(_)));
    if !plain {
        let why = format!("build dir \"{dir}\" is not a relative path inside the folder");
        return BuildDirClone::Refused { dir, why };
    }
    if parent_is_symlink {
        let why = format!("the parent's build dir \"{dir}\" is a symlink; a clone would point into the parent");
        return BuildDirClone::Refused { dir, why };
    }
    let Some(clone) = clone else {
        return BuildDirClone::Absent { dir };
    };
    if in_folder {
        return BuildDirClone::Kept { dir };
    }
    match clone() {
        Ok(()) => BuildDirClone::Shared { dir },
        Err(why) => BuildDirClone::Refused { dir, why },
    }
}
