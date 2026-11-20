use crate::GitError;

fn sample() -> GitError {
    GitError {
        args: vec!["status".to_string()],
        code: Some(128),
        stderr: "fatal: not a git repository".to_string(),
    }
}

#[test]
fn equal_when_every_field_matches() {
    assert_eq!(
        sample(),
        GitError {
            args: vec!["status".to_string()],
            code: Some(128),
            stderr: "fatal: not a git repository".to_string()
        }
    );
}

#[test]
fn differs_on_any_field() {
    assert_ne!(sample(), GitError { args: Vec::new(), ..sample() });
    assert_ne!(sample(), GitError { code: None, ..sample() });
    assert_ne!(sample(), GitError { stderr: String::new(), ..sample() });
}

#[test]
fn is_a_std_error() {
    let _boxed: Box<dyn std::error::Error> = Box::new(sample());
}
