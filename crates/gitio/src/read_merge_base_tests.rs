use crate::read_merge_base::read_merge_base;
use crate::{Oid, Output};

fn output(status: i32, stdout: &[u8], stderr: &[u8]) -> Output {
    Output {
        status,
        stdout: stdout.to_vec(),
        stderr: stderr.to_vec(),
    }
}

#[test]
fn exit_0_is_the_common_ancestor() {
    let out = output(0, b"cf9b8986512445d26313ea6a44990dd9599e8d3c\n", b"");
    assert_eq!(read_merge_base(&out), Ok(Some(Oid::parse("cf9b8986512445d26313ea6a44990dd9599e8d3c").unwrap())));
}

#[test]
fn exit_1_with_empty_output_is_none() {
    assert_eq!(read_merge_base(&output(1, b"", b"")), Ok(None));
}

#[test]
fn exit_0_with_a_bad_line_is_refused() {
    let err = read_merge_base(&output(0, b"cf9b8986512445d26313ea6a44990dd9599e8d3c", b"")).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.stderr.contains("\"cf9b8986512445d26313ea6a44990dd9599e8d3c\""), "{}", err.stderr);
}

#[test]
fn exit_0_with_empty_stdout_is_refused() {
    assert_eq!(read_merge_base(&output(0, b"", b"")).unwrap_err().code, None);
}

#[test]
fn exit_1_with_stderr_is_refused_naming_it() {
    let err = read_merge_base(&output(1, b"", b"error: odd\n")).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert!(err.stderr.contains("exit 1"), "{}", err.stderr);
    assert!(err.stderr.contains("\"error: odd\\n\""), "{}", err.stderr);
}

#[test]
fn exit_1_with_stdout_is_refused_naming_it() {
    let err = read_merge_base(&output(1, b"cf9b8986512445d26313ea6a44990dd9599e8d3c\n", b"")).unwrap_err();
    assert!(err.stderr.contains("\"cf9b8986512445d26313ea6a44990dd9599e8d3c\\n\""), "{}", err.stderr);
}

#[test]
fn other_exit_is_refused_naming_exit_and_stderr() {
    let err = read_merge_base(&output(128, b"", b"fatal: Not a valid object name nope\n")).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.stderr.contains("exit 128"), "{}", err.stderr);
    assert!(err.stderr.contains("\"fatal: Not a valid object name nope\\n\""), "{}", err.stderr);
}
