//! Refusals that are not unmerged paths: IndexTreeError::Git with what ran (or nothing) and git's exit and stderr.
use crate::fixture::repo;
use gitio::{GitError, IndexTreeError};
use std::path::Path;

#[test]
fn an_empty_folder_path_is_refused_before_git_runs() {
    let r = repo();
    let refusal = GitError { args: vec![], code: None, stderr: r#"not a usable worktree path (empty): """#.into() };
    assert_eq!(r.git.index_tree(Path::new("")), Err(IndexTreeError::Git(refusal)));
}
