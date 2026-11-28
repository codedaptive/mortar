use crate::{CommitPathsError, CommitPathsHooksError};
use gitio::HookRefusal;
use std::path::PathBuf;

#[test]
fn commit_paths_hooks_error_shows_one_sentence_per_variant() {
    let rows = [
        (
            CommitPathsHooksError::HookRefused(HookRefusal {
                hook: "pre-commit",
                code: Some(1),
                stdout: b"lint: out\n".to_vec(),
                stderr: b"lint: err\n".to_vec(),
            }),
            "the pre-commit hook refused the commit with exit code 1; no commit was written",
        ),
        (
            CommitPathsHooksError::Paths(CommitPathsError::NotAdded { path: PathBuf::from("ignored.txt") }),
            "the path \"ignored.txt\" is on disk but git ignores it, so it is not in the folder's tree; \
             no commit was written",
        ),
    ];
    for (error, expected) in rows {
        assert_eq!(error.to_string(), expected, "variant {error:?}");
    }
}
