use crate::trailers_add_request::trailers_add_request;
use crate::GitError;
use std::ffi::OsString;

fn pairs(items: &[(&str, &str)]) -> Vec<(String, String)> {
    items.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

fn refused(message: &str, items: &[(&str, &str)]) -> GitError {
    trailers_add_request(message, &pairs(items)).unwrap_err()
}

#[test]
fn each_trailer_is_one_flag_in_order() {
    let (args, _) =
        trailers_add_request("subject\n", &pairs(&[("Change-Id", "abc"), ("Doc-Level", "minor")])).unwrap();
    let expected: Vec<OsString> =
        ["interpret-trailers", "--no-divider", "--trailer", "Change-Id=abc", "--trailer", "Doc-Level=minor"]
            .iter()
            .map(OsString::from)
            .collect();
    assert_eq!(args, expected);
}

#[test]
fn no_trailers_is_the_command_with_no_divider() {
    let (args, _) = trailers_add_request("subject\n", &[]).unwrap();
    assert_eq!(args, vec![OsString::from("interpret-trailers"), OsString::from("--no-divider")]);
}

#[test]
fn value_may_hold_equals_colon_and_spaces() {
    let (args, _) = trailers_add_request("s\n", &pairs(&[("Signed-off-by", "A <a@x> k=v: w")])).unwrap();
    assert_eq!(args[3], OsString::from("Signed-off-by=A <a@x> k=v: w"));
}

#[test]
fn message_goes_to_stdin_read_only() {
    let (_, opts) = trailers_add_request("subject\n\nbody\n", &pairs(&[("K", "v")])).unwrap();
    assert_eq!(opts.stdin, Some(b"subject\n\nbody\n".to_vec()));
    assert!(opts.read_only);
    assert!(opts.answer_codes.is_empty());
    assert!(opts.env.is_empty());
    assert_eq!(opts.cwd, None);
}

#[test]
fn nul_in_message_is_refused() {
    let err = refused("sub\0ject", &[("K", "v")]);
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert!(err.stderr.contains("\"sub\\0ject\""), "{}", err.stderr);
}

#[test]
fn nul_in_key_is_refused() {
    let err = refused("s\n", &[("K\0x", "v")]);
    assert!(err.stderr.contains("\"K\\0x\""), "{}", err.stderr);
}

#[test]
fn nul_in_value_is_refused() {
    let err = refused("s\n", &[("K", "v\0x")]);
    assert!(err.stderr.contains("\"v\\0x\""), "{}", err.stderr);
}

#[test]
fn empty_key_is_refused() {
    let err = refused("s\n", &[("", "v")]);
    assert_eq!(err.code, None);
    assert!(err.stderr.contains("\"\""), "{}", err.stderr);
}

#[test]
fn key_with_equals_is_refused_naming_it() {
    let err = refused("s\n", &[("A=B", "v")]);
    assert!(err.stderr.contains("\"A=B\""), "{}", err.stderr);
}

#[test]
fn key_with_colon_is_refused_naming_it() {
    let err = refused("s\n", &[("A:B", "v")]);
    assert!(err.stderr.contains("\"A:B\""), "{}", err.stderr);
}

#[test]
fn key_with_newline_is_refused() {
    let err = refused("s\n", &[("A\nB", "v")]);
    assert!(err.stderr.contains("\"A\\nB\""), "{}", err.stderr);
}

#[test]
fn key_with_carriage_return_is_refused() {
    assert!(trailers_add_request("s\n", &pairs(&[("A\rB", "v")])).is_err());
}

#[test]
fn value_with_newline_is_refused_naming_it() {
    let err = refused("s\n", &[("K", "one\nTwo: x")]);
    assert!(err.stderr.contains("\"one\\nTwo: x\""), "{}", err.stderr);
}

#[test]
fn value_with_carriage_return_is_refused() {
    assert!(trailers_add_request("s\n", &pairs(&[("K", "one\rtwo")])).is_err());
}

#[test]
fn bad_second_trailer_refuses_the_whole_request() {
    assert!(trailers_add_request("s\n", &pairs(&[("K", "v"), ("", "w")])).is_err());
}
