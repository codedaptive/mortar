use crate::read_rev_parse::read_rev_parse;
use crate::Output;

fn output(status: i32, stdout: &[u8]) -> Output {
    Output {
        status,
        stdout: stdout.to_vec(),
        stderr: Vec::new(),
    }
}

#[test]
fn found_sha1_is_some() {
    let oid = read_rev_parse(&output(0, b"4c3115d7f09202012d6f64a6b6174c6774c3192e\n")).unwrap().unwrap();
    assert_eq!(oid.as_str(), "4c3115d7f09202012d6f64a6b6174c6774c3192e");
}

#[test]
fn found_sha256_is_some() {
    let stdout = b"fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210\n";
    let oid = read_rev_parse(&output(0, stdout)).unwrap().unwrap();
    assert_eq!(oid.as_str(), "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210");
}

#[test]
fn exit_1_with_empty_stdout_is_none() {
    assert_eq!(read_rev_parse(&output(1, b"")), Ok(None));
}

#[test]
fn exit_0_with_bad_stdout_is_refused() {
    assert!(read_rev_parse(&output(0, b"not an oid\n")).is_err());
}

#[test]
fn exit_0_with_empty_stdout_is_refused() {
    assert!(read_rev_parse(&output(0, b"")).is_err());
}

#[test]
fn exit_1_with_stdout_is_refused_naming_it() {
    let err = read_rev_parse(&output(1, b"4c3115d7f09202012d6f64a6b6174c6774c3192e\n")).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert!(err.stderr.contains("exit 1"), "{}", err.stderr);
    assert!(err.stderr.contains("\"4c3115d7f09202012d6f64a6b6174c6774c3192e\\n\""), "{}", err.stderr);
}

#[test]
fn other_exit_is_refused_naming_it() {
    let err = read_rev_parse(&output(2, b"")).unwrap_err();
    assert!(err.stderr.contains("exit 2"), "{}", err.stderr);
}
