//! Oid::as_str: the hex text.

use crate::Oid;

impl Oid {
    /// The id as lowercase hex.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
