//! patch_id_diff_request: the git run that prints one commit's change as the patch text `patch-id` reads.

use crate::{Oid, RunOpts};
use std::ffi::OsString;

/// `diff-tree -p` of `commit`, read-only, with every setting that shapes the patch text fixed.
///
/// `--diff-merges=first-parent` diffs a merge against its first parent (without it diff-tree prints nothing for a
/// merge); `--root` diffs a root commit against the empty tree. The other flags and the environment keep the user's
/// config, environment, replace refs, grafts and worktree attributes from changing the text:
/// - colour, external diff, textconv, renames, relative paths, prefixes, algorithm, indent heuristic, context lines,
///   submodule form and abbreviated blob ids are each set on the command line;
/// - GIT_DIFF_OPTS is empty, as git's own variable overrides even `--unified`;
/// - GIT_CONFIG_COUNT=0 drops config entries passed by count, and GIT_CONFIG_PARAMETERS replaces any inherited `-c`
///   value for this run (git reads it after the count form): quoted paths, no attributes file, blank context lines
///   kept as a space, git's default size past which a file is binary, and the graft warning off;
/// - GIT_NO_REPLACE_OBJECTS and GIT_GRAFT_FILE=/dev/null ignore replace refs and grafts;
/// - GIT_ATTR_NOSYSTEM and GIT_ATTR_SOURCE read attributes from the commit's tree, not the system or the worktree.
///
/// Still shaping it: the repository's own `.git/info/attributes`, and a shallow boundary (a boundary commit reads as
/// a root). The commit is an Oid, hex only, so it cannot be read as an option.
pub(crate) fn patch_id_diff_request(commit: &Oid) -> (Vec<OsString>, RunOpts) {
    let mut args: Vec<OsString> = [
        "diff-tree",
        "-p",
        "--root",
        "--diff-merges=first-parent",
        "--no-color",
        "--no-ext-diff",
        "--no-textconv",
        "--no-renames",
        "--no-relative",
        "--src-prefix=a/",
        "--dst-prefix=b/",
        "--diff-algorithm=myers",
        "--indent-heuristic",
        "--unified=3",
        "--ignore-submodules=none",
        "--submodule=short",
        "--full-index",
    ]
    .map(OsString::from)
    .to_vec();
    args.push(commit.as_str().into());
    let env = [
        ("GIT_DIFF_OPTS", ""),
        ("GIT_CONFIG_COUNT", "0"),
        ("GIT_CONFIG_PARAMETERS", PARAMETERS),
        ("GIT_NO_REPLACE_OBJECTS", "1"),
        ("GIT_GRAFT_FILE", "/dev/null"),
        ("GIT_ATTR_NOSYSTEM", "1"),
        ("GIT_ATTR_SOURCE", commit.as_str()),
    ]
    .map(|(key, value)| (OsString::from(key), OsString::from(value)))
    .to_vec();
    (
        args,
        RunOpts {
            env,
            read_only: true,
            ..RunOpts::default()
        },
    )
}

const PARAMETERS: &str = "'core.quotePath'='true' 'core.attributesFile'='/dev/null' 'diff.suppressBlankEmpty'='false' \
'core.bigFileThreshold'='512m' 'advice.graftFileDeprecated'='false'";
