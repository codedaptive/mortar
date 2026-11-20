use crate::check_exit::check_exit;
use crate::{GitError, Output};

fn args() -> Vec<String> {
    vec!["merge-tree".to_string(), "--write-tree".to_string()]
}

#[test]
fn exit_zero_is_ok_with_the_bytes() {
    let out = check_exit(&args(), Some(0), b"tree\n".to_vec(), b"note".to_vec(), &[]);
    assert_eq!(
        out,
        Ok(Output {
            status: 0,
            stdout: b"tree\n".to_vec(),
            stderr: b"note".to_vec()
        })
    );
}

#[test]
fn exit_one_is_an_error_by_default() {
    let out = check_exit(&args(), Some(1), Vec::new(), b"conflict".to_vec(), &[]);
    assert_eq!(
        out,
        Err(GitError {
            args: args(),
            code: Some(1),
            stderr: "conflict".to_string()
        })
    );
}

#[test]
fn exit_one_is_ok_when_it_is_an_answer_code() {
    let out = check_exit(&args(), Some(1), b"t".to_vec(), Vec::new(), &[1]);
    assert_eq!(
        out,
        Ok(Output {
            status: 1,
            stdout: b"t".to_vec(),
            stderr: Vec::new()
        })
    );
}

#[test]
fn exit_not_in_answer_codes_is_an_error() {
    let out = check_exit(&args(), Some(2), Vec::new(), Vec::new(), &[1]);
    assert_eq!(out.map_err(|e| e.code), Err(Some(2)));
}

#[test]
fn no_exit_code_is_an_error_even_with_answer_codes() {
    let out = check_exit(&args(), None, Vec::new(), b"signal".to_vec(), &[0, 1]);
    assert_eq!(
        out,
        Err(GitError {
            args: args(),
            code: None,
            stderr: "signal".to_string()
        })
    );
}

#[test]
fn error_stderr_is_lossy_utf8() {
    let out = check_exit(&args(), Some(128), Vec::new(), vec![b'b', b'a', b'd', 0xff], &[]);
    assert_eq!(out.map_err(|e| e.stderr), Err("bad\u{FFFD}".to_string()));
}
