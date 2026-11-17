//! GitError's Display text as std::error::Error shows it.
use gitio::GitError;

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
