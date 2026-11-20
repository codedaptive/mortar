//! Reads one string member of a reply's report.

use super::Reply;
use crate::json::JsonValue;

impl Reply {
    /// The member `name` when it is a string; "" when it is missing, null or of another kind.
    pub fn text(&self, name: &str) -> String {
        match self.report.field(name) {
            Some(JsonValue::Str(text)) => text.clone(),
            _ => String::new(),
        }
    }
}
