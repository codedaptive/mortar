//! The value of an extra --json member of a report.
//! A bare type (S-7); render_json_tests exercises each variant.

/// Text and Texts are JSON strings and an array of strings; Number a non-negative integer. Raw is an array of JSON
/// values already encoded (each one whole, as its encoder wrote it), written as they are.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutValue {
    Text(String),
    Number(u64),
    Bool(bool),
    Null,
    Texts(Vec<String>),
    Raw(Vec<String>),
}
