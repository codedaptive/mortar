//! Why text is not a JSON value: it stops early, or it is wrong.
//! A bare type; parse_json_tests and parse_json_string_tests exercise each variant.

/// `EndOfInput`: the text ends before the value is complete, which is what a torn write leaves.
/// `Invalid`: anything else, with what is wrong.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum JsonError {
    EndOfInput,
    Invalid(String),
}
