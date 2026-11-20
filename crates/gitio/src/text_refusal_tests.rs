use crate::text_refusal::text_refusal;

#[test]
fn message_is_what_then_the_quoted_text() {
    let err = text_refusal("trailer line without ':'", "x");
    assert!(err.args.is_empty());
    assert_eq!(err.code, None);
    assert_eq!(err.stderr, "trailer line without ':': \"x\"");
}

#[test]
fn line_breaks_nuls_and_quotes_are_escaped() {
    let err = text_refusal("ref line without a space", "a\nb\0c\"d");
    assert_eq!(err.stderr, "ref line without a space: \"a\\nb\\0c\\\"d\"");
}

#[test]
fn empty_text_shows_as_empty_quotes() {
    assert_eq!(text_refusal("not a usable trailer key", "").stderr, "not a usable trailer key: \"\"");
}
