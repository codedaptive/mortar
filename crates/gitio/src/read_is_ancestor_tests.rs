use crate::read_is_ancestor::read_is_ancestor;
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
    assert_eq!(read_is_ancestor(&output(0, b"", b"")), Ok(true));
}

#[test]
fn exit_1_with_empty_output_is_false() {
    assert_eq!(read_is_ancestor(&output(1, b"", b"")), Ok(false));
}

#[test]
fn exit_0_with_stdout_is_refused_naming_it() {
    let err = read_is_ancestor(&output(0, b"cf9b8986512445d26313ea6a44990dd9599e8d3c\n", b"")).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert_eq!(
        err.stderr,
        "merge-base --is-ancestor answered exit 0 with stdout \"cf9b8986512445d26313ea6a44990dd9599e8d3c\\n\" and stderr \"\""
    );
}

#[test]
fn exit_0_with_stderr_is_refused_naming_it() {
    let err = read_is_ancestor(&output(0, b"", b"warning: odd\n")).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.stderr.contains("exit 0"), "{}", err.stderr);
    assert!(err.stderr.contains("\"warning: odd\\n\""), "{}", err.stderr);
}

#[test]
fn exit_1_with_stderr_is_refused_not_false() {
    let err = read_is_ancestor(&output(1, b"", b"error: odd\n")).unwrap_err();
    assert_eq!(err.code, None);
    assert_eq!(err.stderr, "merge-base --is-ancestor answered exit 1 with stdout \"\" and stderr \"error: odd\\n\"");
}

#[test]
fn exit_1_with_stdout_is_refused_not_false() {
    let err = read_is_ancestor(&output(1, b"x\n", b"")).unwrap_err();
    assert!(err.stderr.contains("\"x\\n\""), "{}", err.stderr);
}

#[test]
fn other_exit_is_refused_naming_exit_and_stderr() {
    let stderr = b"fatal: Not a valid commit name 1111111111111111111111111111111111111111\n";
    let err = read_is_ancestor(&output(128, b"", stderr)).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.stderr.contains("exit 128"), "{}", err.stderr);
    assert!(err.stderr.contains("fatal: Not a valid commit name 1111111111111111111111111111111111111111\\n"), "{}", err.stderr);
}
