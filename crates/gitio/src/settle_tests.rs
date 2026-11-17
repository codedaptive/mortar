use crate::settle::settle;
use crate::{GitError, Output};
use std::io;
use std::path::Path;

fn shown() -> Vec<String> {
    vec!["hash-object".to_string(), "--stdin".to_string()]
}

fn answer() -> Output {
    Output {
        status: 0,
        stdout: b"ok\n".to_vec(),
        stderr: Vec::new(),
    }
}

fn exit_error() -> GitError {
    GitError {
        args: shown(),
        code: Some(128),
        stderr: "fatal: bad object".to_string(),
    }
}

#[test]
fn settle_keeps_the_exit_error_over_a_stdin_failure() {
    let other = io::Error::other("disk gone");
    assert_eq!(settle(Err(exit_error()), Err(other), Path::new("git"), &shown()), Err(exit_error()));
    let broken = io::Error::from(io::ErrorKind::BrokenPipe);
    assert_eq!(settle(Err(exit_error()), Err(broken), Path::new("git"), &shown()), Err(exit_error()));
}

#[test]
fn settle_keeps_the_answer_after_a_broken_pipe() {
    let broken = io::Error::from(io::ErrorKind::BrokenPipe);
    assert_eq!(settle(Ok(answer()), Err(broken), Path::new("git"), &shown()), Ok(answer()));
}

#[test]
fn settle_refuses_an_answer_after_another_stdin_failure() {
    let other = io::Error::other("disk gone");
    assert_eq!(
        settle(Ok(answer()), Err(other), Path::new("/usr/bin/git"), &shown()),
        Err(GitError {
            args: vec!["hash-object".to_string(), "--stdin".to_string()],
            code: None,
            stderr: "could not write stdin to /usr/bin/git: disk gone".to_string(),
        })
    );
}

#[test]
fn settle_keeps_the_answer_when_stdin_was_fed() {
    assert_eq!(settle(Ok(answer()), Ok(()), Path::new("git"), &shown()), Ok(answer()));
}
