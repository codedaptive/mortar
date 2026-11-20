//! PatchId::as_str: the hex text.

use crate::PatchId;

impl PatchId {
    /// The patch ID as lowercase hex.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
