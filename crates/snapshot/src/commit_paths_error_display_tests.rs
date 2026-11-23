use crate::CommitPathsError;
use gitio::{GitError, IndexTreeError, Oid, TreeWithPathsRefusal};
use std::io::{Error, ErrorKind};
use std::path::PathBuf;

fn path(text: &str) -> PathBuf {
    PathBuf::from(text)
}

#[test]
fn commit_paths_error_shows_one_sentence_per_variant() {
    let git = GitError { args: vec!["write-tree".to_string()], code: Some(128), stderr: "fatal: bad tree\n".to_string() };
    let commit = Oid::parse("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap();
    let rows: Vec<(CommitPathsError, &str)> = vec![
        (
            CommitPathsError::Absolute { path: path("/w/x") },
            "the path \"/w/x\" is absolute; paths are named relative to the folder; nothing was written",
        ),
        (
            CommitPathsError::ParentPart { path: path("../x") },
            "the path \"../x\" has a '..' part, which could leave the folder; nothing was written",
        ),
        (
            CommitPathsError::ThroughSymlink { path: path("link/x"), link: path("link") },
            "the path \"link/x\" goes through the symlink \"link\", which could leave the folder; nothing was written",
        ),
        (
            CommitPathsError::Ancestor { path: path("a/b/c"), error: Error::new(ErrorKind::PermissionDenied, "denied") },
            "a folder on the way to the path \"a/b/c\" could not be looked at: denied; nothing was written",
        ),
        (
            CommitPathsError::NotAdded { path: path("ignored.txt") },
            "the path \"ignored.txt\" is on disk but git ignores it, so it is not in the folder's tree; \
             no commit was written",
        ),
        (
            CommitPathsError::Paths(TreeWithPathsRefusal::FolderInFrom { path: path("src") }),
            "the paths are refused: not a file entry: \"src\" is a folder; no commit was written",
        ),
        (
            CommitPathsError::Git(git),
            "a git run failed: git write-tree: exit code 128: fatal: bad tree; no commit was written",
        ),
        (
            CommitPathsError::IndexNotSet { commit, error: IndexTreeError::Unmerged { paths: vec![path("a.txt")] } },
            "commit aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa was written but the folder's index could not be set for \
             its paths: index has unmerged paths: \"a.txt\"; no ref moved, the files are untouched and the index is \
             the old one, or the merged tree without its skip-worktree bits if the sparse-checkout reapply failed",
        ),
    ];
    for (error, expected) in rows {
        assert_eq!(error.to_string(), expected, "variant {error:?}");
    }
}
