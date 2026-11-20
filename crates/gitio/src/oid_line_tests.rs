use crate::oid_line::oid_line;

#[test]
fn sha1_line_is_the_oid() {
    let oid = oid_line(b"4c3115d7f09202012d6f64a6b6174c6774c3192e\n").unwrap();
    assert_eq!(oid.as_str(), "4c3115d7f09202012d6f64a6b6174c6774c3192e");
}

#[test]
fn sha256_line_is_the_oid() {
    let oid = oid_line(b"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\n").unwrap();
    assert_eq!(oid.as_str(), "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef");
}

#[test]
fn missing_newline_is_refused_naming_the_text() {
    let err = oid_line(b"4c3115d7f09202012d6f64a6b6174c6774c3192e").unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.stderr.contains("\"4c3115d7f09202012d6f64a6b6174c6774c3192e\""), "{}", err.stderr);
}

#[test]
fn empty_output_is_refused() {
    assert!(oid_line(b"").is_err());
}

#[test]
fn two_newlines_are_refused() {
    assert!(oid_line(b"4c3115d7f09202012d6f64a6b6174c6774c3192e\n\n").is_err());
}

#[test]
fn two_lines_are_refused() {
    assert!(oid_line(b"4c3115d7f09202012d6f64a6b6174c6774c3192e\n4c3115d7f09202012d6f64a6b6174c6774c3192e\n").is_err());
}

#[test]
fn short_hex_is_refused() {
    assert!(oid_line(b"4c3115d\n").is_err());
}

#[test]
fn crlf_is_refused() {
    assert!(oid_line(b"4c3115d7f09202012d6f64a6b6174c6774c3192e\r\n").is_err());
}
