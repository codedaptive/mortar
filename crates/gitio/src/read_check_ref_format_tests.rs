use crate::read_check_ref_format::read_check_ref_format;
use crate::Output;

fn output(status: i32, stdout: &[u8], stderr: &[u8]) -> Output {
    Output {
        status,
        stdout: stdout.to_vec(),
        stderr: stderr.to_vec(),
    }
}

#[test]
fn exit_0_with_empty_output_is_true() {
    assert_eq!(read_check_ref_format(&output(0, b"", b"")), Ok(true));
}

#[test]
fn exit_1_with_empty_output_is_false() {
    assert_eq!(read_check_ref_format(&output(1, b"", b"")), Ok(false));
}

#[test]
fn exit_0_with_stdout_is_refused_naming_it() {
    let err = read_check_ref_format(&output(0, b"refs/heads/main\n", b"")).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert!(err.stderr.contains("exit 0"), "{}", err.stderr);
    assert!(err.stderr.contains("\"refs/heads/main\\n\""), "{}", err.stderr);
}

#[test]
fn exit_1_with_stderr_is_refused_naming_it() {
    let err = read_check_ref_format(&output(1, b"", b"fatal: odd\n")).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.stderr.contains("exit 1"), "{}", err.stderr);
    assert!(err.stderr.contains("\"fatal: odd\\n\""), "{}", err.stderr);
}

#[test]
fn exit_1_with_stdout_is_refused_naming_it() {
    let err = read_check_ref_format(&output(1, b"x\n", b"")).unwrap_err();
    assert!(err.stderr.contains("\"x\\n\""), "{}", err.stderr);
}

#[test]
fn usage_exit_129_is_refused_naming_exit_and_stderr() {
    let err = read_check_ref_format(&output(129, b"", b"usage: git check-ref-format\n")).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.stderr.contains("exit 129"), "{}", err.stderr);
    assert!(err.stderr.contains("\"usage: git check-ref-format\\n\""), "{}", err.stderr);
}
