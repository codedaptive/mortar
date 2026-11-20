//! The value of an extra member of a report: a --json member, or the savings, which also opens the plain form.
//! A bare type; render_json_tests exercises each variant.

use crate::savings::Savings;

/// Text and Texts are JSON strings and an array of strings; Number a non-negative integer. Raw is an array of JSON
/// values already encoded (each one whole, as its encoder wrote it), written as they are. Saved is the savings of an
/// answer, rendered as the plain "[mr] tokens saved" line and the --json object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutValue {
    Text(String),
    Number(u64),
    Bool(bool),
    Null,
    Texts(Vec<String>),
    Raw(Vec<String>),
    Saved(Savings),
}
