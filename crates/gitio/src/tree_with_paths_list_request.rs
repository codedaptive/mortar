//! tree_with_paths_list_request: the git run that lists the entries a set of paths names in a tree.

use crate::usable_path::usable_path;
use crate::{GitError, Oid, RunOpts};
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

/// `-c core.precomposeunicode=false ls-tree -z --full-tree --end-of-options <from> -- <paths...>`, read-only, with
/// GIT_LITERAL_PATHSPECS=1 so each path names an entry, never a pattern (see tree_entry_request); the paths go into
/// argv as their bytes. None for no paths: ls-tree with no path would list the whole root. Each path names one entry,
/// matched as its bytes, because the paths that would not are refused and precomposition is off:
/// - an empty path or one holding a NUL is refused naming it (usable_path);
/// - a path ending in '/' is refused naming it: it names a folder's children rather than an entry;
/// - a path not written as git stores it (a leading '/', an empty part, a "." or ".." part) is refused naming it: git
///   rewrites such a path before matching, or answers with an error;
/// - two paths where one is a folder prefix of the other (`p` and `p/...`) are refused naming both: ls-tree lists
///   every child of the folder `p`. The same path twice passes.
///
/// With core.precomposeunicode on (macOS's default) git rewrites argv paths to precomposed Unicode, so a decomposed
/// name would not be found. Line breaks and tabs pass: the output is -z.
pub(crate) fn tree_with_paths_list_request(
    from: &Oid,
    paths: &[PathBuf],
) -> Result<Option<(Vec<OsString>, RunOpts)>, GitError> {
    if paths.is_empty() {
        return Ok(None);
    }
    for path in paths {
        usable_path(path)?;
        stored_form(path)?;
    }
    no_folder_prefix(paths)?;
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

/// Refuses, naming `path`, a path ending in '/', starting with '/', or with an empty, "." or ".." part.
fn stored_form(path: &Path) -> Result<(), GitError> {
    let bytes = path.as_os_str().as_bytes();
    if bytes.ends_with(b"/") {
        return Err(GitError::refusal(
            Vec::new(),
            format!("not a usable path (ends in '/', naming a folder's children): {path:?}"),
        ));
    }
    if bytes.split(|b| *b == b'/').any(|part| matches!(part, b"" | b"." | b"..")) {
        return Err(GitError::refusal(
            Vec::new(),
            format!("not a usable path (not as git stores it: a leading '/', or an empty, '.' or '..' part): {path:?}"),
        ));
    }
    Ok(())
}

/// Refuses, naming both, any two paths where one equals the other's bytes up to a '/'.
fn no_folder_prefix(paths: &[PathBuf]) -> Result<(), GitError> {
    for (i, a) in paths.iter().enumerate() {
        for b in &paths[i + 1..] {
            let (folder, inner) = if a.as_os_str().len() <= b.as_os_str().len() { (a, b) } else { (b, a) };
            let (short, long) = (folder.as_os_str().as_bytes(), inner.as_os_str().as_bytes());
            if long.len() > short.len() && long.starts_with(short) && long[short.len()] == b'/' {
                return Err(GitError::refusal(
                    Vec::new(),
                    format!("not usable paths (one is a folder holding the other): {folder:?} and {inner:?}"),
                ));
            }
        }
    }
    Ok(())
}
