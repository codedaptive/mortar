use crate::read_patch_id::read_patch_id;
use crate::{Oid, PatchId};

const COMMIT: &str = "4c3115d7f09202012d6f64a6b6174c6774c3192e";

fn commit() -> Oid {
    Oid::parse(COMMIT).unwrap()
}

fn refused(stdout: &[u8]) -> String {
    let err = read_patch_id(stdout, &commit()).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    err.stderr
}

#[test]
fn empty_stdout_is_none() {
    assert_eq!(read_patch_id(b"", &commit()), Ok(None));
}

#[test]
fn one_line_naming_the_commit_is_its_patch_id() {
    let out = b"f4b5bd8a1c3e2d9e7a6b5c4d3e2f1a0b9c8d7e6f 4c3115d7f09202012d6f64a6b6174c6774c3192e\n";
    assert_eq!(
        read_patch_id(out, &commit()),
        Ok(Some(PatchId::parse("f4b5bd8a1c3e2d9e7a6b5c4d3e2f1a0b9c8d7e6f").unwrap()))
    );
}

#[test]
fn a_64_hex_line_is_read() {
    let id = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let commit = Oid::parse("fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210").unwrap();
    let out = format!("{id} fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210\n");
    assert_eq!(
        read_patch_id(out.as_bytes(), &commit).unwrap().unwrap().as_str(),
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
    );
}

#[test]
fn two_lines_are_refused_naming_them() {
    let msg = refused(
        b"1111111111111111111111111111111111111111 4c3115d7f09202012d6f64a6b6174c6774c3192e\n\
          2222222222222222222222222222222222222222 4c3115d7f09202012d6f64a6b6174c6774c3192e\n",
    );
    assert!(msg.contains("4c3115d7f09202012d6f64a6b6174c6774c3192e\\n2222222222222222222222222222222222222222"), "{msg}");
}

#[test]
fn a_missing_newline_is_refused() {
    let msg = refused(b"1111111111111111111111111111111111111111 4c3115d7f09202012d6f64a6b6174c6774c3192e");
    assert_eq!(
        msg,
        "patch-id answered \"1111111111111111111111111111111111111111 4c3115d7f09202012d6f64a6b6174c6774c3192e\", \
         not one \"<patch id> 4c3115d7f09202012d6f64a6b6174c6774c3192e\" line"
    );
}

#[test]
fn an_extra_field_is_refused() {
    let msg = refused(b"1111111111111111111111111111111111111111 4c3115d7f09202012d6f64a6b6174c6774c3192e x\n");
    assert!(msg.contains("4c3115d7f09202012d6f64a6b6174c6774c3192e x\\n"), "{msg}");
}

#[test]
fn a_single_field_is_refused() {
    let msg = refused(b"1111111111111111111111111111111111111111\n");
    assert!(msg.contains("\"1111111111111111111111111111111111111111\\n\""), "{msg}");
}

#[test]
fn bad_hex_is_refused() {
    let msg = refused(b"111111111111111111111111111111111111111g 4c3115d7f09202012d6f64a6b6174c6774c3192e\n");
    assert!(msg.contains("111111111111111111111111111111111111111g"), "{msg}");
}

#[test]
fn an_uppercase_patch_id_is_refused() {
    let msg = refused(b"ABCDEF1111111111111111111111111111111111 4c3115d7f09202012d6f64a6b6174c6774c3192e\n");
    assert!(msg.contains("ABCDEF1111111111111111111111111111111111"), "{msg}");
}

#[test]
fn a_different_commit_id_is_refused() {
    let msg = refused(b"1111111111111111111111111111111111111111 0000000000000000000000000000000000000000\n");
    assert!(msg.contains("1111111111111111111111111111111111111111 0000000000000000000000000000000000000000"), "{msg}");
}

#[test]
fn non_utf8_is_refused_with_lossy_text() {
    let msg = refused(b"\xff1111111111111111111111111111111111111111 4c3115d7f09202012d6f64a6b6174c6774c3192e\n");
    assert!(msg.contains("\u{fffd}1111111111111111111111111111111111111111"), "{msg}");
}
