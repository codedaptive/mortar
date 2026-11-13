use crate::read_message::read_message;

#[test]
fn utf8_is_kept_unchanged() {
    assert_eq!(
        read_message(b"subject\n\nbody caf\xc3\xa9\n\nChange-Id: abc\n").unwrap(),
        "subject\n\nbody caf\u{e9}\n\nChange-Id: abc\n"
    );
}

#[test]
fn trailers_added_to_an_empty_message() {
    assert_eq!(read_message(b"\nK: v\n").unwrap(), "\nK: v\n");
}

#[test]
fn empty_output_is_empty() {
    assert_eq!(read_message(b"").unwrap(), "");
}

#[test]
fn non_utf8_is_refused_naming_the_bytes() {
    let err = read_message(b"caf\xe9\n").unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert!(err.stderr.contains("\"caf\u{fffd}\\n\""), "{}", err.stderr);
}
