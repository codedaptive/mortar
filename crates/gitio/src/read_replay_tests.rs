use crate::read_replay::read_replay;
use crate::{GitError, Oid, Output, RefUpdate, ReplayOutcome};

fn output(status: i32, stdout: &[u8], stderr: &[u8]) -> Output {
    Output {
        status,
        stdout: stdout.to_vec(),
        stderr: stderr.to_vec(),
    }
}

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn captured_clean_replay_is_one_update_with_its_old_value() {
    let stdout = b"update refs/heads/rc f5e4a7a76cb0b2823f3e70e37133c90217e72d60 f67e59d9bd3426d1433ea69e2257f6dfb660acc3\n";
    assert_eq!(
        read_replay(&output(0, stdout, b"")),
        Ok(ReplayOutcome::Replayed(vec![RefUpdate {
            refname: "refs/heads/rc".to_string(),
            new: oid("f5e4a7a76cb0b2823f3e70e37133c90217e72d60"),
            old: Some(oid("f67e59d9bd3426d1433ea69e2257f6dfb660acc3")),
        }]))
    );
}

#[test]
fn two_lines_with_64_hex_ids_keep_their_order() {
    let stdout = b"update refs/heads/b 0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789\nupdate refs/heads/a f5e4a7a76cb0b2823f3e70e37133c90217e72d60 f67e59d9bd3426d1433ea69e2257f6dfb660acc3\n";
    assert_eq!(
        read_replay(&output(0, stdout, b"")),
        Ok(ReplayOutcome::Replayed(vec![
            RefUpdate {
                refname: "refs/heads/b".to_string(),
                new: oid("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"),
                old: Some(oid("abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789")),
            },
            RefUpdate {
                refname: "refs/heads/a".to_string(),
                new: oid("f5e4a7a76cb0b2823f3e70e37133c90217e72d60"),
                old: Some(oid("f67e59d9bd3426d1433ea69e2257f6dfb660acc3")),
            },
        ]))
    );
}

#[test]
fn exit_0_with_no_output_is_replayed_with_no_updates() {
    assert_eq!(read_replay(&output(0, b"", b"")), Ok(ReplayOutcome::Replayed(vec![])));
}

#[test]
fn exit_1_with_empty_stdout_is_conflicted() {
    assert_eq!(read_replay(&output(1, b"", b"")), Ok(ReplayOutcome::Conflicted));
}

#[test]
fn exit_1_with_stderr_is_gits_error_not_a_conflict() {
    let out = output(1, b"", b"git: 'replay' is not a git command. See 'git --help'.\n");
    assert_eq!(
        read_replay(&out),
        Err(GitError {
            args: Vec::new(),
            code: Some(1),
            stderr: "git: 'replay' is not a git command. See 'git --help'.\n".to_string(),
        })
    );
}

#[test]
fn exit_1_with_output_is_refused_naming_it() {
    let stdout = b"update refs/heads/rc f5e4a7a76cb0b2823f3e70e37133c90217e72d60 f67e59d9bd3426d1433ea69e2257f6dfb660acc3\n";
    let err = read_replay(&output(1, stdout, b"")).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert!(err.stderr.contains("exit 1"), "{}", err.stderr);
    assert!(err.stderr.contains("update refs/heads/rc"), "{}", err.stderr);
}

#[test]
fn other_exit_is_refused_naming_it() {
    let err = read_replay(&output(2, b"", b"")).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.stderr.contains("exit 2"), "{}", err.stderr);
}

#[test]
fn line_with_another_command_is_refused_naming_it() {
    let stdout = b"delete refs/heads/rc f67e59d9bd3426d1433ea69e2257f6dfb660acc3\n";
    let err = read_replay(&output(0, stdout, b"")).unwrap_err();
    assert_eq!(err.code, None);
    assert_eq!(
        err.stderr,
        "not a replay update line: \"delete refs/heads/rc f67e59d9bd3426d1433ea69e2257f6dfb660acc3\""
    );
}

#[test]
fn line_without_an_old_value_is_refused() {
    let stdout = b"update refs/heads/rc f5e4a7a76cb0b2823f3e70e37133c90217e72d60\n";
    let err = read_replay(&output(0, stdout, b"")).unwrap_err();
    assert!(err.stderr.starts_with("not a replay update line"), "{}", err.stderr);
}

#[test]
fn line_with_an_extra_field_is_refused() {
    let stdout = b"update refs/heads/rc f5e4a7a76cb0b2823f3e70e37133c90217e72d60 f67e59d9bd3426d1433ea69e2257f6dfb660acc3 x\n";
    assert!(read_replay(&output(0, stdout, b"")).is_err());
}

#[test]
fn empty_refname_is_refused() {
    let stdout = b"update  f5e4a7a76cb0b2823f3e70e37133c90217e72d60 f67e59d9bd3426d1433ea69e2257f6dfb660acc3\n";
    let err = read_replay(&output(0, stdout, b"")).unwrap_err();
    assert!(err.stderr.starts_with("not a replay update line"), "{}", err.stderr);
}

#[test]
fn bad_new_id_is_refused_naming_it() {
    let stdout = b"update refs/heads/rc f5e4a7a f67e59d9bd3426d1433ea69e2257f6dfb660acc3\n";
    let err = read_replay(&output(0, stdout, b"")).unwrap_err();
    assert!(err.stderr.contains("\"f5e4a7a\""), "{}", err.stderr);
}

#[test]
fn bad_old_id_is_refused_naming_it() {
    let stdout = b"update refs/heads/rc f5e4a7a76cb0b2823f3e70e37133c90217e72d60 F67E\n";
    let err = read_replay(&output(0, stdout, b"")).unwrap_err();
    assert!(err.stderr.contains("\"F67E\""), "{}", err.stderr);
}

#[test]
fn output_without_its_final_newline_is_refused() {
    let stdout = b"update refs/heads/rc f5e4a7a76cb0b2823f3e70e37133c90217e72d60 f67e59d9bd3426d1433ea69e2257f6dfb660acc3";
    let err = read_replay(&output(0, stdout, b"")).unwrap_err();
    assert!(err.stderr.starts_with("replay output without its final newline"), "{}", err.stderr);
}

#[test]
fn non_utf8_output_is_refused() {
    let err = read_replay(&output(0, b"update refs/heads/caf\xe9 a b\n", b"")).unwrap_err();
    assert!(err.stderr.contains("not UTF-8"), "{}", err.stderr);
}
