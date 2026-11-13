use crate::read_show_ref::read_show_ref;
use crate::{GitError, Oid, Output};

const MAIN: &str = "6269f0e88f79b3eada64287692f6d3f03196c4a7";

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
fn found_full_ref_is_some() {
    let out = output(0, b"6269f0e88f79b3eada64287692f6d3f03196c4a7 refs/heads/main\n", b"");
    assert_eq!(read_show_ref(&out, "refs/heads/main"), Ok(Some(oid(MAIN))));
}

#[test]
fn symbolic_ref_prints_the_targets_id_under_the_name_asked() {
    let out = output(0, b"6269f0e88f79b3eada64287692f6d3f03196c4a7 refs/heads/sym\n", b"");
    assert_eq!(read_show_ref(&out, "refs/heads/sym"), Ok(Some(oid(MAIN))));
}

#[test]
fn head_is_some() {
    let out = output(0, b"6269f0e88f79b3eada64287692f6d3f03196c4a7 HEAD\n", b"");
    assert_eq!(read_show_ref(&out, "HEAD"), Ok(Some(oid(MAIN))));
}

#[test]
fn sha256_id_is_some() {
    let out = output(0, b"fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210 refs/tags/v1\n", b"");
    assert_eq!(
        read_show_ref(&out, "refs/tags/v1"),
        Ok(Some(oid("fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210")))
    );
}

#[test]
fn not_a_valid_ref_for_the_name_sent_is_none() {
    let out = output(128, b"", b"fatal: 'refs/heads/gone' - not a valid ref\n");
    assert_eq!(read_show_ref(&out, "refs/heads/gone"), Ok(None));
}

#[test]
fn not_a_valid_ref_for_a_short_name_is_none() {
    let out = output(128, b"", b"fatal: 'main' - not a valid ref\n");
    assert_eq!(read_show_ref(&out, "main"), Ok(None));
}

#[test]
fn other_128_is_gits_error_with_its_stderr() {
    let stderr = b"fatal: not a git repository (or any of the parent directories): .git\n";
    assert_eq!(
        read_show_ref(&output(128, b"", stderr), "refs/heads/main"),
        Err(GitError {
            args: Vec::new(),
            code: Some(128),
            stderr: "fatal: not a git repository (or any of the parent directories): .git\n".to_string(),
        })
    );
}

#[test]
fn not_a_valid_ref_for_another_name_is_gits_error() {
    let err = read_show_ref(&output(128, b"", b"fatal: 'refs/heads/other' - not a valid ref\n"), "refs/heads/main")
        .unwrap_err();
    assert_eq!(err.code, Some(128));
    assert_eq!(err.stderr, "fatal: 'refs/heads/other' - not a valid ref\n");
}

#[test]
fn not_a_valid_ref_without_its_newline_is_gits_error() {
    let err = read_show_ref(&output(128, b"", b"fatal: 'refs/heads/main' - not a valid ref"), "refs/heads/main")
        .unwrap_err();
    assert_eq!(err.code, Some(128));
}

#[test]
fn not_a_valid_ref_with_stdout_is_gits_error() {
    let out = output(128, b"x\n", b"fatal: 'refs/heads/main' - not a valid ref\n");
    assert_eq!(read_show_ref(&out, "refs/heads/main").unwrap_err().code, Some(128));
}

#[test]
fn exit_0_for_another_name_is_refused_naming_stdout() {
    let out = output(0, b"6269f0e88f79b3eada64287692f6d3f03196c4a7 refs/heads/main\n", b"");
    let err = read_show_ref(&out, "refs/heads/other").unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert!(
        err.stderr.contains("\"6269f0e88f79b3eada64287692f6d3f03196c4a7 refs/heads/main\\n\""),
        "{}",
        err.stderr
    );
}

#[test]
fn exit_0_with_a_bad_id_is_refused_naming_stdout() {
    let err = read_show_ref(&output(0, b"not-an-oid refs/heads/main\n", b""), "refs/heads/main").unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.stderr.contains("\"not-an-oid refs/heads/main\\n\""), "{}", err.stderr);
}

#[test]
fn exit_0_without_the_final_newline_is_refused() {
    let out = output(0, b"6269f0e88f79b3eada64287692f6d3f03196c4a7 refs/heads/main", b"");
    assert_eq!(read_show_ref(&out, "refs/heads/main").unwrap_err().code, None);
}

#[test]
fn exit_0_with_two_lines_is_refused() {
    let out = output(
        0,
        b"6269f0e88f79b3eada64287692f6d3f03196c4a7 refs/heads/main\n6269f0e88f79b3eada64287692f6d3f03196c4a7 refs/heads/main\n",
        b"",
    );
    assert_eq!(read_show_ref(&out, "refs/heads/main").unwrap_err().code, None);
}

#[test]
fn exit_0_with_empty_stdout_is_refused() {
    assert_eq!(read_show_ref(&output(0, b"", b""), "refs/heads/main").unwrap_err().code, None);
}

#[test]
fn exit_1_is_refused_naming_exit_and_stdout() {
    let err = read_show_ref(&output(1, b"odd\n", b""), "refs/heads/main").unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.stderr.contains("exit 1"), "{}", err.stderr);
    assert!(err.stderr.contains("\"odd\\n\""), "{}", err.stderr);
}
