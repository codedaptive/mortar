//! commit_paths_error_report: every CommitPathsError variant, and every TreeWithPathsRefusal inside Paths, in the
//! cli's own words, none of snapshot's or gitio's texts.

use std::path::PathBuf;

use gitio::{GitError, IndexTreeError, Oid, TreeWithPathsRefusal};
use snapshot::CommitPathsError;

use super::commit_paths_error_report;
use crate::report::{ExitCode, Report};

const INSIDE: &str = "name paths inside the folder, relative to its top";
const FILES: &str = "name the files inside the folder, not the folder that holds them";

fn path(text: &str) -> PathBuf {
    PathBuf::from(text)
}

fn refused(line: &str, next: &str) -> Report {
    Report {
        verb: "save".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![line.to_string(), "nothing was written and no ref moved".to_string()],
        owner: None,
        next: next.to_string(),
        fields: Vec::new(),
    }
}

fn failed(line: &str) -> Report {
    Report {
        verb: "save".to_string(),
        exit: ExitCode::Failure,
        outcome: "failed".to_string(),
        lines: vec![line.to_string()],
        owner: None,
        next: "repeat the same command".to_string(),
        fields: Vec::new(),
    }
}

#[test]
fn each_error_is_reported_in_the_clis_words() {
    let commit = Oid::parse("1111111111111111111111111111111111111111").unwrap();
    let paths = CommitPathsError::Paths;
    for (error, expected) in [
        (
            CommitPathsError::Absolute { path: path("/etc/x") },
            refused("/etc/x is absolute; paths are named relative to the folder", INSIDE),
        ),
        (
            CommitPathsError::ParentPart { path: path("a/../b") },
            refused("a/../b has a .. part, which could leave the folder", INSIDE),
        ),
        (
            CommitPathsError::ThroughSymlink { path: path("l/f"), link: path("l") },
            refused("l/f is reached through the symlinked folder l", INSIDE),
        ),
        (
            CommitPathsError::NotAdded { path: path("out.log") },
            refused("git ignores out.log, so it cannot be committed", "name paths git tracks or does not ignore"),
        ),
        (
            paths(TreeWithPathsRefusal::FolderInFrom { path: path("src") }),
            refused("src is a folder in the folder", FILES),
        ),
        (
            paths(TreeWithPathsRefusal::FolderInBase { path: path("old") }),
            refused("old is gone from the folder and is a folder in the commit it builds on", FILES),
        ),
        (
            paths(TreeWithPathsRefusal::Overlap { folder: path("p"), inner: path("p/q") }),
            refused("p is named, and so is p/q inside it", FILES),
        ),
        (
            paths(TreeWithPathsRefusal::NotAsStored { path: path("a/") }),
            refused(
                "a/ is not written as git stores it: it has a leading or trailing /, or an empty, . or .. part",
                INSIDE,
            ),
        ),
        (
            paths(TreeWithPathsRefusal::Unusable { path: path("") }),
            refused("the path \"\" is empty or holds a NUL", INSIDE),
        ),
        (
            CommitPathsError::Ancestor { path: path("a/b"), error: std::io::Error::other("io text") },
            failed("git or the file system failed while reading the named paths"),
        ),
        (
            CommitPathsError::Git(GitError::refusal(Vec::new(), "gitio text")),
            failed("git or the file system failed while reading the named paths"),
        ),
        (
            CommitPathsError::IndexNotSet { commit, error: IndexTreeError::Unmerged { paths: vec![path("x")] } },
            failed(
                "commit 1111111111111111111111111111111111111111 was written but the folder's index was not set \
                 for the named paths; no ref moved",
            ),
        ),
    ] {
        assert_eq!(commit_paths_error_report("save", &error), expected, "{error:?}");
    }
}
