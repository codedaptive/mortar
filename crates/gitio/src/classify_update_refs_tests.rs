use crate::classify_update_refs::classify_update_refs;
use crate::{GitError, Oid, Output, RefUpdate, UpdateRefsOutcome};

const NEW: &str = "4c3115d7f09202012d6f64a6b6174c6774c3192e";
const STALE_X: &str = "fatal: prepare: cannot lock ref 'refs/heads/x': is at fbb9d6dc3b2f2d6e61bbac74bd2725eb193a6d9a but expected 4c3115d7f09202012d6f64a6b6174c6774c3192e\n";

fn update(refname: &str) -> RefUpdate {
    RefUpdate {
        refname: refname.to_string(),
        new: Oid::parse(NEW).unwrap(),
        old: None,
    }
}

fn failed(stderr: &str) -> GitError {
    GitError {
        args: vec!["update-ref".to_string(), "--stdin".to_string(), "-z".to_string()],
        code: Some(128),
        stderr: stderr.to_string(),
    }
}

fn stale(refname: &str) -> Result<UpdateRefsOutcome, GitError> {
    Ok(UpdateRefsOutcome::Stale {
        refname: refname.to_string(),
    })
}

fn unchanged(stderr: &str, refnames: &[&str]) {
    let updates: Vec<RefUpdate> = refnames.iter().map(|name| update(name)).collect();
    assert_eq!(classify_update_refs(Err(failed(stderr)), &updates), Err(failed(stderr)));
}

#[test]
fn success_is_updated() {
    let ok = Output {
        status: 0,
        stdout: b"start: ok\nprepare: ok\ncommit: ok\n".to_vec(),
        stderr: Vec::new(),
    };
    assert_eq!(classify_update_refs(Ok(ok), &[update("refs/heads/x")]), Ok(UpdateRefsOutcome::Updated));
}

#[test]
fn is_at_another_value_is_stale() {
    assert_eq!(classify_update_refs(Err(failed(STALE_X)), &[update("refs/heads/x")]), stale("refs/heads/x"));
}

#[test]
fn existing_ref_expected_absent_is_stale() {
    let stderr = "fatal: prepare: cannot lock ref 'refs/heads/x': reference already exists\n";
    assert_eq!(classify_update_refs(Err(failed(stderr)), &[update("refs/heads/x")]), stale("refs/heads/x"));
}

#[test]
fn missing_ref_expected_present_is_stale() {
    let stderr = "fatal: prepare: cannot lock ref 'refs/heads/nope': unable to resolve reference 'refs/heads/nope'\n";
    assert_eq!(classify_update_refs(Err(failed(stderr)), &[update("refs/heads/nope")]), stale("refs/heads/nope"));
}

#[test]
fn missing_but_expected_wording_is_stale() {
    let stderr = "fatal: prepare: cannot lock ref 'refs/heads/x': reference is missing but expected 4c3115d7f09202012d6f64a6b6174c6774c3192e\n";
    assert_eq!(classify_update_refs(Err(failed(stderr)), &[update("refs/heads/x")]), stale("refs/heads/x"));
}

#[test]
fn second_update_stale_names_it_not_the_first() {
    let updates = [update("refs/heads/q"), update("refs/heads/x")];
    assert_eq!(classify_update_refs(Err(failed(STALE_X)), &updates), stale("refs/heads/x"));
}

#[test]
fn refname_with_a_quote_is_matched_whole() {
    let stderr = "fatal: prepare: cannot lock ref 'refs/heads/it's': unable to resolve reference 'refs/heads/it's'\n";
    assert_eq!(classify_update_refs(Err(failed(stderr)), &[update("refs/heads/it's")]), stale("refs/heads/it's"));
}

#[test]
fn held_lock_is_an_error() {
    unchanged(
        "fatal: prepare: cannot lock ref 'refs/heads/x': Unable to create '/tmp/r/.git/refs/heads/x.lock': File exists.\n\nAnother git process seems to be running in this repository, or the lock file may be stale\n",
        &["refs/heads/x"],
    );
}

#[test]
fn directory_file_clash_is_an_error() {
    unchanged(
        "fatal: prepare: cannot lock ref 'refs/heads/x/sub': 'refs/heads/x' exists; cannot create 'refs/heads/x/sub'\n",
        &["refs/heads/x/sub"],
    );
}

#[test]
fn bad_ref_name_is_an_error() {
    unchanged("fatal: invalid ref format: refs/heads/a..b\n", &["refs/heads/a..b"]);
}

#[test]
fn missing_object_is_an_error() {
    unchanged(
        "fatal: trying to write ref 'refs/heads/w' with nonexistent object deadbeefdeadbeefdeadbeefdeadbeefdeadbeef\n",
        &["refs/heads/w"],
    );
}

#[test]
fn duplicate_ref_is_an_error() {
    unchanged(
        "fatal: prepare: multiple updates for ref 'refs/heads/m' not allowed\n",
        &["refs/heads/m", "refs/heads/m"],
    );
}

#[test]
fn refusal_without_exit_code_is_an_error_even_with_stale_text() {
    let refusal = GitError::refusal(Vec::new(), STALE_X);
    assert_eq!(
        classify_update_refs(Err(refusal), &[update("refs/heads/x")]),
        Err(GitError {
            args: Vec::new(),
            code: None,
            stderr: STALE_X.to_string()
        })
    );
}

#[test]
fn stale_text_for_a_ref_not_sent_is_an_error() {
    unchanged(STALE_X, &["refs/heads/q"]);
}

#[test]
fn broken_ref_is_an_error() {
    unchanged(
        "fatal: prepare: cannot lock ref 'refs/heads/broken': unable to resolve reference 'refs/heads/broken': reference broken\n",
        &["refs/heads/broken"],
    );
}

#[test]
fn unreadable_ref_is_an_error() {
    unchanged(
        "fatal: prepare: cannot lock ref 'refs/heads/p': unable to resolve reference 'refs/heads/p': Permission denied\n",
        &["refs/heads/p"],
    );
}

#[test]
fn missing_ref_at_end_of_stderr_is_stale() {
    let stderr = "fatal: prepare: cannot lock ref 'refs/heads/nope': unable to resolve reference 'refs/heads/nope'";
    assert_eq!(classify_update_refs(Err(failed(stderr)), &[update("refs/heads/nope")]), stale("refs/heads/nope"));
}
