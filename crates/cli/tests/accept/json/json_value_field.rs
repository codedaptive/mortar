//! Looks up a member of a JSON object.

use super::JsonValue;

impl JsonValue {
    /// The value of the first member named `name`; None when there is none or this is not an object.
    pub fn field(&self, name: &str) -> Option<&JsonValue> {
        match self {
            JsonValue::Object(members) => members.iter().find(|(key, _)| key == name).map(|(_, value)| value),
            _ => None,
        }
    }
}
