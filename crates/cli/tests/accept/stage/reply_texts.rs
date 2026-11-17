//! Reads one string-list member of a reply's report.

use super::Reply;
use crate::json::JsonValue;

impl Reply {
    /// The strings of the array member `name`, in order, any item that is not a string left out; none when the
    /// member is missing or not an array.
    pub fn texts(&self, name: &str) -> Vec<String> {
        match self.report.field(name) {
            Some(JsonValue::Array(items)) => items
                .iter()
                .filter_map(|item| match item {
                    JsonValue::Str(text) => Some(text.clone()),
                    _ => None,
                })
                .collect(),
            _ => Vec::new(),
        }
    }
}
