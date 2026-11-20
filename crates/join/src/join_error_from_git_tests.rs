use crate::JoinError;
use gitio::GitError;

fn failing() -> Result<(), GitError> {
    Err(GitError { args: vec!["replay".to_string()], code: Some(1), stderr: "fatal: no".to_string() })
}

fn through_question_mark() -> Result<(), JoinError> {
    failing()?;
    Ok(())
}

#[test]
fn git_error_becomes_the_git_variant_unchanged() {
    assert_eq!(
        through_question_mark(),
        Err(JoinError::Git(GitError {
            args: vec!["replay".to_string()],
            code: Some(1),
            stderr: "fatal: no".to_string()
        }))
    );
}
