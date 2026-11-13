use crate::read_replay_probe::read_replay_probe;
use crate::{GitError, Output};

const USAGE_256: &[u8] = b"usage: git replay ([--contained] --onto <newbase> | --advance <branch>) <revision-range>\n\n    --ref-action <mode>   control ref update behavior (update|print)\n";
const USAGE_OLD: &[u8] =
    b"usage: (EXPERIMENTAL!) git replay ([--contained] --onto <newbase> | --advance <branch>) <revision-range>...\n";

fn output(status: i32, stdout: &[u8], stderr: &[u8]) -> Result<Output, GitError> {
    Ok(Output {
        status,
        stdout: stdout.to_vec(),
        stderr: stderr.to_vec(),
    })
}

#[test]
fn usage_on_stdout_naming_ref_action_is_true() {
    assert_eq!(read_replay_probe(output(0, USAGE_256, b"")), Ok(true));
}

#[test]
fn usage_on_stderr_naming_ref_action_is_true() {
    assert_eq!(read_replay_probe(output(129, b"", USAGE_256)), Ok(true));
}

#[test]
fn usage_without_ref_action_is_false() {
    assert_eq!(read_replay_probe(output(129, b"", USAGE_OLD)), Ok(false));
}

#[test]
fn no_output_is_false() {
    assert_eq!(read_replay_probe(output(0, b"", b"")), Ok(false));
}

#[test]
fn git_without_replay_is_false() {
    let err = GitError {
        args: vec!["replay".to_string(), "-h".to_string()],
        code: Some(1),
        stderr: "git: 'replay' is not a git command. See 'git --help'.\n".to_string(),
    };
    assert_eq!(read_replay_probe(Err(err)), Ok(false));
}

#[test]
fn other_errors_are_returned_unchanged() {
    let err = GitError {
        args: vec!["replay".to_string(), "-h".to_string()],
        code: None,
        stderr: "could not start git: No such file or directory".to_string(),
    };
    assert_eq!(read_replay_probe(Err(err.clone())), Err(err));
}
