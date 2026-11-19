use crate::GitError;

#[test]
fn shows_args_exit_code_and_stderr() {
    let err = GitError {
        args: vec!["merge-tree".to_string(), "--write-tree".to_string()],
        code: Some(2),
        stderr: "fatal: bad revision\n".to_string(),
    };
    assert_eq!(
        err.to_string(),
        "git merge-tree --write-tree: exit code 2: fatal: bad revision"
    );
}

#[test]
fn shows_no_exit_code_when_code_is_none() {
    let err = GitError {
        args: vec!["fetch".to_string()],
        code: None,
        stderr: "killed".to_string(),
    };
    assert_eq!(err.to_string(), "git fetch: no exit code: killed");
}

#[test]
fn omits_the_args_segment_when_args_are_empty() {
    let refused = GitError {
        args: Vec::new(),
        code: None,
        stderr: "not a git object id".to_string(),
    };
    assert_eq!(refused.to_string(), "git: no exit code: not a git object id");
    let exited = GitError {
        args: Vec::new(),
        code: Some(128),
        stderr: "fatal".to_string(),
    };
    assert_eq!(exited.to_string(), "git: exit code 128: fatal");
}

fn sample() -> GitError {
    GitError {
        args: vec!["status".to_string()],
        code: Some(128),
        stderr: "fatal: not a git repository".to_string(),
    }
}

#[test]
fn is_a_std_error_showing_its_text() {
    let boxed: Box<dyn std::error::Error> = Box::new(sample());
    assert_eq!(
        boxed.to_string(),
        "git status: exit code 128: fatal: not a git repository"
    );
}
