use crate::read_symbolic_ref::read_symbolic_ref;
use crate::Output;

fn output(status: i32, stdout: &[u8], stderr: &[u8]) -> Output {
    Output {
        status,
        stdout: stdout.to_vec(),
        stderr: stderr.to_vec(),
    }
}

#[test]
fn exit_0_is_the_branch() {
    assert_eq!(read_symbolic_ref(&output(0, b"refs/heads/main\n", b"")), Ok(Some("refs/heads/main".to_string())));
}

#[test]
fn exit_1_with_empty_output_is_detached() {
    assert_eq!(read_symbolic_ref(&output(1, b"", b"")), Ok(None));
}

#[test]
fn non_utf8_name_is_refused_naming_it() {
    let err = read_symbolic_ref(&output(0, b"refs/heads/caf\xe9\n", b"")).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert!(err.stderr.contains("\"refs/heads/caf\u{fffd}\\n\""), "{}", err.stderr);
}

#[test]
fn empty_line_is_refused_naming_it() {
    let err = read_symbolic_ref(&output(0, b"\n", b"")).unwrap_err();
    assert_eq!(err.code, None);
    assert_eq!(err.stderr, "not one symbolic-ref line: \"\\n\"");
}

#[test]
fn empty_stdout_on_exit_0_is_refused() {
    let err = read_symbolic_ref(&output(0, b"", b"")).unwrap_err();
    assert_eq!(err.stderr, "not one symbolic-ref line: \"\"");
}

#[test]
fn two_lines_are_refused_naming_them() {
    let err = read_symbolic_ref(&output(0, b"refs/heads/main\nrefs/heads/side\n", b"")).unwrap_err();
    assert_eq!(err.code, None);
    assert_eq!(err.stderr, "not one symbolic-ref line: \"refs/heads/main\\nrefs/heads/side\\n\"");
}

#[test]
fn missing_final_newline_is_refused_naming_it() {
    let err = read_symbolic_ref(&output(0, b"refs/heads/main", b"")).unwrap_err();
    assert_eq!(err.code, None);
    assert_eq!(err.stderr, "not one symbolic-ref line: \"refs/heads/main\"");
}

#[test]
fn exit_1_with_stdout_is_refused_naming_it() {
    let err = read_symbolic_ref(&output(1, b"refs/heads/main\n", b"")).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert!(err.stderr.contains("exit 1"), "{}", err.stderr);
    assert!(err.stderr.contains("\"refs/heads/main\\n\""), "{}", err.stderr);
}

#[test]
fn exit_1_with_stderr_is_refused_naming_it() {
    let err = read_symbolic_ref(&output(1, b"", b"error: odd\n")).unwrap_err();
    assert!(err.stderr.contains("exit 1"), "{}", err.stderr);
    assert!(err.stderr.contains("\"error: odd\\n\""), "{}", err.stderr);
}

#[test]
fn other_exit_is_refused_naming_it() {
    let err = read_symbolic_ref(&output(128, b"", b"fatal: not a git repository\n")).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.stderr.contains("exit 128"), "{}", err.stderr);
    assert!(err.stderr.contains("\"fatal: not a git repository\\n\""), "{}", err.stderr);
}
