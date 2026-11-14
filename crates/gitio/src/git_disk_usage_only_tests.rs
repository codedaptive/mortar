use crate::Git;
use std::path::PathBuf;

/// A Git whose program does not exist: any run it attempted would end in a spawn refusal ("could not run ...").
fn no_git() -> Git {
    Git {
        program: PathBuf::from("/nonexistent/wo-gitio-12/git"),
        repo: PathBuf::from("/nonexistent/wo-gitio-12/repo"),
        env: Vec::new(),
    }
}

#[test]
fn empty_list_is_zero_without_running_git() {
    assert_eq!(no_git().disk_usage_only::<&str>(&[]), Ok(0));
}
