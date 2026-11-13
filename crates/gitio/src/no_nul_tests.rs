use crate::no_nul::no_nul;

#[test]
fn text_without_nul_passes() {
    assert_eq!(no_nul("rev", b"refs/heads/main"), Ok(()));
}

#[test]
fn empty_text_passes() {
    assert_eq!(no_nul("rev", b""), Ok(()));
}

#[test]
fn nul_is_refused_naming_what_and_the_text() {
    let err = no_nul("trailer key", b"Change\0Id").unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert_eq!(err.stderr, "trailer key holds a NUL: \"Change\\0Id\"");
}

#[test]
fn trailing_nul_is_refused() {
    let err = no_nul("message", b"subject\n\0").unwrap_err();
    assert_eq!(err.stderr, "message holds a NUL: \"subject\\n\\0\"");
}
