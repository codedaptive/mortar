//! attempts_dir: where `mr folder attempts <n> <dir>` makes its attempts (S-127, SPEC-cli 05): `dir` taken from
//! the request folder's toplevel when relative, normalized lexically, and refused when it is the toplevel or inside it.

use std::path::{Component, Path, PathBuf};

/// `dir` joined to `toplevel` when relative (an absolute `dir` stays as it is), then normalized without touching the
/// filesystem, since `dir` need not exist yet: `.` components dropped, each `..` popping the component before it.
/// Err(reason) when the result is `toplevel` or lies inside it; the reason names both paths.
pub fn attempts_dir(toplevel: &Path, dir: &Path) -> Result<PathBuf, String> {
    let mut resolved = PathBuf::new();
    for component in toplevel.join(dir).components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                resolved.pop();
            }
            other => resolved.push(other),
        }
    }
    if resolved.starts_with(toplevel) {
        return Err(format!(
            "<dir> {} is inside the request folder {}; attempts go outside it",
            resolved.display(),
            toplevel.display()
        ));
    }
    Ok(resolved)
}
