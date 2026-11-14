use crate::classify_fetch_ref::classify_fetch_ref;
use crate::{FetchRefOutcome, GitError, Output};

const MISSING: &str = "fatal: couldn't find remote ref refs/heads/nosuch\n";

/// git's error for a failed fetch, built from a refusal so the test names only the fields it is about.
fn failed(code: Option<i32>, stderr: &str) -> GitError {
    let mut err = GitError::refusal(vec!["fetch".to_string(), "--no-tags".to_string()], stderr);
    err.code = code;
    err
}

#[test]
fn success_is_fetched() {
    let ok = Output {
        status: 0,
        stdout: Vec::new(),
        stderr: b"From ../remote\n * [new branch]      main       -> x/main\n".to_vec(),
    };
    assert_eq!(classify_fetch_ref(Ok(ok), "refs/heads/main"), Ok(FetchRefOutcome::Fetched));
}

#[test]
fn couldnt_find_remote_ref_is_missing_source() {
    assert_eq!(
        classify_fetch_ref(Err(failed(Some(128), MISSING)), "refs/heads/nosuch"),
        Ok(FetchRefOutcome::MissingSource)
    );
}

#[test]
fn missing_wording_for_another_ref_is_returned_unchanged() {
    assert_eq!(
        classify_fetch_ref(Err(failed(Some(128), MISSING)), "refs/heads/main"),
        Err(failed(Some(128), MISSING))
    );
}

#[test]
fn missing_wording_with_more_after_it_is_returned_unchanged() {
    let stderr = "fatal: couldn't find remote ref refs/heads/nosuch\nfatal: the remote end hung up\n";
    assert_eq!(
        classify_fetch_ref(Err(failed(Some(128), stderr)), "refs/heads/nosuch"),
        Err(failed(Some(128), stderr))
    );
}

#[test]
fn missing_wording_under_another_code_is_returned_unchanged() {
    assert_eq!(
        classify_fetch_ref(Err(failed(Some(1), MISSING)), "refs/heads/nosuch"),
        Err(failed(Some(1), MISSING))
    );
}

#[test]
fn refusal_with_the_missing_wording_is_returned_unchanged() {
    assert_eq!(
        classify_fetch_ref(Err(failed(None, MISSING)), "refs/heads/nosuch"),
        Err(failed(None, MISSING))
    );
}

#[test]
fn remote_that_is_not_a_repository_keeps_gits_error() {
    let stderr = "fatal: '../nosuch.git' does not appear to be a git repository\nfatal: Could not read from remote repository.\n\nPlease make sure you have the correct access rights\nand the repository exists.\n";
    assert_eq!(
        classify_fetch_ref(Err(failed(Some(128), stderr)), "refs/heads/nosuch"),
        Err(failed(Some(128), stderr))
    );
}
