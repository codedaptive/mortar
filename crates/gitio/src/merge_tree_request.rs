//! merge_tree_request: the git run that merges two commits into a tree without touching any folder.

use crate::{Oid, RunOpts};
use std::ffi::OsString;

/// `--attr-source=<attr_source> merge-tree --write-tree -z --name-only [--merge-base=<b>] <ours> <theirs>`.
/// `--attr-source` is a git option, not a merge-tree one, so it comes before the command: merge attributes are read
/// from that commit, as there is no folder to read them from. Exit 1 is the conflict answer. merge-tree writes tree
/// objects, so it is not read-only.
pub(crate) fn merge_tree_request(
    attr_source: &Oid,
    merge_base: Option<&Oid>,
    ours: &Oid,
    theirs: &Oid,
) -> (Vec<OsString>, RunOpts) {
    let mut args: Vec<OsString> = vec![
        format!("--attr-source={}", attr_source.as_str()).into(),
        "merge-tree".into(),
        "--write-tree".into(),
        "-z".into(),
        "--name-only".into(),
    ];
    if let Some(base) = merge_base {
        args.push(format!("--merge-base={}", base.as_str()).into());
    }
    args.push(ours.as_str().into());
    args.push(theirs.as_str().into());
    (
        args,
        RunOpts {
            answer_codes: vec![1],
            ..RunOpts::default()
        },
    )
}
