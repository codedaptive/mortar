use crate::read_ls_remote::read_ls_remote;
use crate::Oid;

const MAIN: &str = "fb863af4e7dd149507b87a188f4b02b2e0093eb2";
const TAG: &str = "c604f71d204fe327f8675e5282299b9259d357d4";
const PEELED: &str = "19a3b7dabeab83f72877c0e05c3960937a59ae67";

fn pair(hex: &str, name: &str) -> (Oid, String) {
    (Oid::parse(hex).unwrap(), name.to_string())
}

#[test]
fn one_ref_line_is_one_pair() {
    let got = read_ls_remote(b"fb863af4e7dd149507b87a188f4b02b2e0093eb2\trefs/heads/main\n").unwrap();
    assert_eq!(got, vec![pair(MAIN, "refs/heads/main")]);
}

#[test]
fn annotated_tag_keeps_its_peeled_line_in_order() {
    let stdout = b"c604f71d204fe327f8675e5282299b9259d357d4\trefs/tags/v1\n19a3b7dabeab83f72877c0e05c3960937a59ae67\trefs/tags/v1^{}\n";
    assert_eq!(
        read_ls_remote(stdout).unwrap(),
        vec![pair(TAG, "refs/tags/v1"), pair(PEELED, "refs/tags/v1^{}")]
    );
}

#[test]
fn sha256_ids_are_read() {
    let hex = "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210";
    let got = read_ls_remote(format!("{hex}\tHEAD\n").as_bytes()).unwrap();
    assert_eq!(got, vec![pair(hex, "HEAD")]);
}

#[test]
fn empty_output_is_no_refs() {
    assert_eq!(read_ls_remote(b"").unwrap(), Vec::new());
}

#[test]
fn missing_final_newline_is_refused_naming_the_text() {
    let err = read_ls_remote(b"fb863af4e7dd149507b87a188f4b02b2e0093eb2\trefs/heads/main").unwrap_err();
    assert_eq!(err.code, None);
    assert_eq!(
        err.stderr,
        "ls-remote lines without their final newline: \"fb863af4e7dd149507b87a188f4b02b2e0093eb2\\trefs/heads/main\""
    );
}

#[test]
fn line_without_a_tab_is_refused_naming_it() {
    let err = read_ls_remote(b"fb863af4e7dd149507b87a188f4b02b2e0093eb2 refs/heads/main\n").unwrap_err();
    assert_eq!(err.code, None);
    assert_eq!(err.stderr, "not an ls-remote line: \"fb863af4e7dd149507b87a188f4b02b2e0093eb2 refs/heads/main\"");
}

#[test]
fn line_with_an_empty_name_is_refused() {
    let err = read_ls_remote(b"fb863af4e7dd149507b87a188f4b02b2e0093eb2\t\n").unwrap_err();
    assert_eq!(err.stderr, "not an ls-remote line: \"fb863af4e7dd149507b87a188f4b02b2e0093eb2\\t\"");
}

#[test]
fn line_whose_id_is_not_an_id_is_refused_naming_the_line() {
    let err = read_ls_remote(b"fb863af\trefs/heads/main\n").unwrap_err();
    assert_eq!(err.stderr, "not an ls-remote line: \"fb863af\\trefs/heads/main\"");
}

#[test]
fn empty_line_between_refs_is_refused() {
    let err = read_ls_remote(b"fb863af4e7dd149507b87a188f4b02b2e0093eb2\trefs/heads/main\n\n").unwrap_err();
    assert_eq!(err.stderr, "not an ls-remote line: \"\"");
}
