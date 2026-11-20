use crate::Oid;

const SHA1: &str = "0123456789abcdef0123456789abcdef01234567";
const SHA256: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

#[test]
fn accepts_40_and_64_lowercase_hex() {
    assert_eq!(Oid::parse(SHA1).unwrap().as_str(), SHA1);
    assert_eq!(Oid::parse(SHA256).unwrap().as_str(), SHA256);
}

#[test]
fn refuses_wrong_lengths() {
    for len in [0, 39, 41, 63, 65] {
        let text = "a".repeat(len);
        assert!(Oid::parse(&text).is_err(), "length {len} accepted");
    }
}

#[test]
fn refuses_uppercase_non_hex_and_whitespace() {
    let bad = [
        "0123456789ABCDEF0123456789abcdef01234567",
        "0123456789abcdeg0123456789abcdef01234567",
        "0123456789abcdef0123456789abcdef0123456 ",
        " 0123456789abcdef0123456789abcdef0123456",
        "0123456789abcdef0123456789abcdef01234567\n",
    ];
    for text in bad {
        assert!(Oid::parse(text).is_err(), "{text:?} accepted");
    }
}

#[test]
fn refusal_names_the_text_with_no_code_or_args() {
    let err = Oid::parse("HEAD").unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert!(err.stderr.contains("\"HEAD\""), "{}", err.stderr);
}
