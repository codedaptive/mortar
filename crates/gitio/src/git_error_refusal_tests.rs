use crate::GitError;

#[test]
fn refusal_has_no_code_and_carries_the_message() {
    let err = GitError::refusal(vec!["version".to_string()], "too old");
    assert_eq!(
        err,
        GitError {
            args: vec!["version".to_string()],
            code: None,
            stderr: "too old".to_string()
        }
    );
}

#[test]
fn refusal_takes_an_owned_string() {
    let err = GitError::refusal(Vec::new(), String::from("bad id"));
    assert_eq!(err.stderr, "bad id");
    assert!(err.args.is_empty());
}
