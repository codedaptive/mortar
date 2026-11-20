use crate::PatchId;

#[test]
fn accepts_40_and_64_lowercase_hex() {
    assert_eq!(
        PatchId::parse("0123456789abcdef0123456789abcdef01234567").unwrap().as_str(),
        "0123456789abcdef0123456789abcdef01234567"
    );
    assert_eq!(
        PatchId::parse("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef").unwrap().as_str(),
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
    );
}

#[test]
fn refuses_wrong_lengths() {
    for len in [0, 39, 41, 63, 65] {
        let text = "a".repeat(len);
        assert!(PatchId::parse(&text).is_err(), "length {len} accepted");
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
        assert!(PatchId::parse(text).is_err(), "{text:?} accepted");
    }
}

#[test]
fn refusal_names_a_patch_id_and_the_text() {
    let err = PatchId::parse("HEAD").unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert_eq!(err.stderr, "not a patch id (40 or 64 lowercase hex chars): \"HEAD\"");
}
