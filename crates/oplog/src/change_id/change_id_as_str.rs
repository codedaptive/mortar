//! The change ID's text.

use super::ChangeId;

impl ChangeId {
    /// The 32 chars of k..z.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
