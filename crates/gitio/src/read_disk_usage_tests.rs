use crate::read_disk_usage::read_disk_usage;
use crate::{GitError, Output};

fn output(status: i32, stdout: &[u8], stderr: &[u8]) -> Output {
    Output {
        status,
        stdout: stdout.to_vec(),
        stderr: stderr.to_vec(),
    }
}

fn read(status: i32, stdout: &[u8], stderr: &[u8]) -> Result<u64, GitError> {
    read_disk_usage(&output(status, stdout, stderr), &["refs/heads/side"])
}

#[test]
fn a_count_is_its_number() {
    assert_eq!(read(0, b"20246\n", b""), Ok(20246));
}

#[test]
fn zero_is_zero() {
    assert_eq!(read(0, b"0\n", b""), Ok(0));
}

#[test]
fn u64_max_is_read() {
    assert_eq!(read(0, b"18446744073709551615\n", b""), Ok(u64::MAX));
}

#[test]
fn exit_0_ignores_stderr() {
    assert_eq!(read(0, b"66\n", b"warning: something\n"), Ok(66));
}

#[test]
fn exit_0_with_anything_but_one_digit_line_is_refused_naming_exit_and_stdout() {
    for (stdout, shown) in [
        ("", "\"\""),
        ("20246", "\"20246\""),
        ("20246 \n", "\"20246 \\n\""),
        ("+5\n", "\"+5\\n\""),
        ("-1\n", "\"-1\\n\""),
        ("abc\n", "\"abc\\n\""),
        ("1\n2\n", "\"1\\n2\\n\""),
        ("18446744073709551616\n", "\"18446744073709551616\\n\""),
        ("\n", "\"\\n\""),
    ] {
        let err = read(0, stdout.as_bytes(), b"").unwrap_err();
        assert_eq!(err.code, None, "{shown}");
        assert!(err.stderr.contains("exit 0"), "{}", err.stderr);
        assert!(err.stderr.contains(&format!("stdout {shown}")), "{}", err.stderr);
    }
}

#[test]
fn bad_revision_for_the_second_sent_ref_names_it() {
    let out = output(128, b"", b"fatal: bad revision 'refs/heads/nope'\n");
    let err = read_disk_usage(&out, &["refs/heads/side", "refs/heads/nope"]).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.stderr.contains("\"refs/heads/nope\""), "{}", err.stderr);
    assert!(!err.stderr.contains("side"), "{}", err.stderr);
}

#[test]
fn bad_revision_for_a_name_with_a_quote_names_it() {
    let out = output(128, b"", b"fatal: bad revision 'refs/heads/n'o'\n");
    let err = read_disk_usage(&out, &["refs/heads/n'o"]).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.stderr.contains("\"refs/heads/n'o\""), "{}", err.stderr);
}

#[test]
fn bad_revision_for_a_ref_not_sent_is_gits_error() {
    assert_eq!(
        read(128, b"", b"fatal: bad revision 'refs/heads/other'\n"),
        Err(GitError {
            args: Vec::new(),
            code: Some(128),
            stderr: "fatal: bad revision 'refs/heads/other'\n".to_string(),
        })
    );
}

#[test]
fn bad_revision_with_stdout_is_gits_error() {
    let err = read(128, b"x\n", b"fatal: bad revision 'refs/heads/side'\n").unwrap_err();
    assert_eq!(err.code, Some(128));
}

#[test]
fn not_a_repository_is_gits_error() {
    assert_eq!(
        read(128, b"", b"fatal: not a git repository (or any of the parent directories): .git\n"),
        Err(GitError {
            args: Vec::new(),
            code: Some(128),
            stderr: "fatal: not a git repository (or any of the parent directories): .git\n".to_string(),
        })
    );
}

#[test]
fn corrupt_given_ref_is_gits_error() {
    let stderr = b"warning: ignoring broken ref refs/heads/bad\nfatal: bad revision 'refs/heads/bad'\n";
    let err = read_disk_usage(&output(128, b"", stderr), &["refs/heads/bad"]).unwrap_err();
    assert_eq!(err.code, Some(128));
    assert_eq!(
        err.stderr,
        "warning: ignoring broken ref refs/heads/bad\nfatal: bad revision 'refs/heads/bad'\n"
    );
}

#[test]
fn corrupt_other_ref_is_gits_error() {
    let err = read_disk_usage(&output(128, b"", b"fatal: bad object refs/heads/bad\n"), &["refs/heads/good"])
        .unwrap_err();
    assert_eq!(err.code, Some(128));
    assert_eq!(err.stderr, "fatal: bad object refs/heads/bad\n");
}

#[test]
fn exit_1_is_refused_naming_exit_stdout_and_stderr() {
    let err = read(1, b"odd\n", b"why\n").unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.stderr.contains("exit 1"), "{}", err.stderr);
    assert!(err.stderr.contains("\"odd\\n\""), "{}", err.stderr);
    assert!(err.stderr.contains("\"why\\n\""), "{}", err.stderr);
}
