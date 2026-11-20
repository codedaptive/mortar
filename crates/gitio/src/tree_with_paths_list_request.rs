//! tree_with_paths_list_request: the git run that lists the entries a set of paths names in a tree.

use crate::usable_path::usable_path;
use crate::{Oid, RunOpts, TreeWithPathsError, TreeWithPathsRefusal};
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

/// `-c core.precomposeunicode=false ls-tree -z --full-tree --end-of-options <from> -- <paths...>`, read-only, with
/// GIT_LITERAL_PATHSPECS=1 so each path names an entry, never a pattern (see tree_entry_request); the paths go into
/// argv as their bytes. None for no paths: ls-tree with no path would list the whole root. Each path names one entry,
/// matched as its bytes, because the paths that would not are refused and precomposition is off:
/// - an empty path or one holding a NUL is Unusable (usable_path);
/// - a path ending in '/' is NotAsStored: it names a folder's children rather than an entry;
/// - a path not written as git stores it (a leading '/', an empty part, a "." or ".." part) is NotAsStored: git
///   rewrites such a path before matching, or answers with an error;
/// - two paths where one is a folder prefix of the other (`p` and `p/...`) are an Overlap naming both, the shorter as
///   `folder`: ls-tree lists every child of the folder `p`. The same path twice passes.
///
/// Each path is checked in order for Unusable then NotAsStored before any Overlap. With core.precomposeunicode on
/// (macOS's default) git rewrites argv paths to precomposed Unicode, so a decomposed name would not be found. Line
/// breaks and tabs pass: the output is -z.
pub(crate) fn tree_with_paths_list_request(
    from: &Oid,
    paths: &[PathBuf],
) -> Result<Option<(Vec<OsString>, RunOpts)>, TreeWithPathsError> {
    if paths.is_empty() {
        return Ok(None);
    }
    for path in paths {
        if usable_path(path).is_err() {
            return Err(TreeWithPathsError::Refused(TreeWithPathsRefusal::Unusable { path: path.clone() }));
        }
        if !stored_form(path) {
            return Err(TreeWithPathsError::Refused(TreeWithPathsRefusal::NotAsStored { path: path.clone() }));
        }
    }
    if let Some((folder, inner)) = folder_prefix(paths) {
        return Err(TreeWithPathsError::Refused(TreeWithPathsRefusal::Overlap {
            folder: folder.clone(),
            inner: inner.clone(),
        }));
    }
    let mut args: Vec<OsString> = [
        "-c",
        "core.precomposeunicode=false",
        "ls-tree",
        "-z",
        "--full-tree",
        "--end-of-options",
        from.as_str(),
        "--",
    ]
    .map(OsString::from)
    .to_vec();
    args.extend(paths.iter().map(OsString::from));
    let opts = RunOpts {
        env: vec![("GIT_LITERAL_PATHSPECS".into(), "1".into())],
        read_only: true,
        ..RunOpts::default()
    };
    Ok(Some((args, opts)))
}

/// False for a path ending in '/', or one starting with '/' or with an empty, "." or ".." part.
fn stored_form(path: &Path) -> bool {
    let bytes = path.as_os_str().as_bytes();
    !bytes.ends_with(b"/") && !bytes.split(|b| *b == b'/').any(|part| matches!(part, b"" | b"." | b".."))
}

/// The first two paths, shorter first, where one equals the other's bytes up to a '/'.
fn folder_prefix(paths: &[PathBuf]) -> Option<(&PathBuf, &PathBuf)> {
    for (i, a) in paths.iter().enumerate() {
        for b in &paths[i + 1..] {
            let (folder, inner) = if a.as_os_str().len() <= b.as_os_str().len() { (a, b) } else { (b, a) };
            let (short, long) = (folder.as_os_str().as_bytes(), inner.as_os_str().as_bytes());
            if long.len() > short.len() && long.starts_with(short) && long[short.len()] == b'/' {
                return Some((folder, inner));
            }
        }
    }
    None
}
