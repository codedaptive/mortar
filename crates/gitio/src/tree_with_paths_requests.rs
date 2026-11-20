//! tree_with_paths_requests: the three git runs that build a tree from a base tree and some paths' entries through a
//! separate index.

use crate::{Oid, RunOpts};
use std::ffi::OsString;
use std::path::Path;

/// `read-tree --end-of-options <base>`, then `update-index -z --index-info` fed `stdin` (index_info_stdin's bytes;
/// git takes --index-info as its last argument, so -z comes before it), then `write-tree`, each with
/// GIT_INDEX_FILE=<index> so the folder's own index is never read or written. --index-info takes its input with add,
/// replace and remove allowed, so a path's entry replaces any base entry it collides with as file versus folder. None
/// is read-only: each writes the index or objects.
pub(crate) fn tree_with_paths_requests(base: &Oid, index: &Path, stdin: Vec<u8>) -> [(Vec<OsString>, RunOpts); 3] {
    let opts = RunOpts {
        env: vec![("GIT_INDEX_FILE".into(), index.as_os_str().to_os_string())],
        ..RunOpts::default()
    };
    [
        (["read-tree", "--end-of-options", base.as_str()].map(OsString::from).to_vec(), opts.clone()),
        (
            ["update-index", "-z", "--index-info"].map(OsString::from).to_vec(),
            RunOpts {
                stdin: Some(stdin),
                ..opts.clone()
            },
        ),
        (vec!["write-tree".into()], opts),
    ]
}
