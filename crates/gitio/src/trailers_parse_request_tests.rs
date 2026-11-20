use crate::trailers_parse_request::trailers_parse_request;
use std::ffi::OsString;

#[test]
fn argv_is_parse_with_no_divider() {
    let (args, _) = trailers_parse_request("subject\n\nChange-Id: abc\n").unwrap();
    assert_eq!(
        args,
        vec![OsString::from("interpret-trailers"), OsString::from("--parse"), OsString::from("--no-divider")]
    );
}

#[test]
fn message_goes_to_stdin_as_given_read_only() {
    let (_, opts) = trailers_parse_request("subject\n\nChange-Id: abc\n").unwrap();
    assert_eq!(opts.stdin, Some(b"subject\n\nChange-Id: abc\n".to_vec()));
    assert!(opts.read_only);
    assert!(opts.answer_codes.is_empty());
    assert!(opts.env.is_empty());
    assert_eq!(opts.cwd, None);
}

#[test]
fn empty_message_is_empty_stdin() {
    let (_, opts) = trailers_parse_request("").unwrap();
    assert_eq!(opts.stdin, Some(Vec::new()));
}

#[test]
fn nul_in_message_is_refused_naming_it() {
    let err = trailers_parse_request("subject\0more").unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert!(err.stderr.contains("\"subject\\0more\""), "{}", err.stderr);
}
