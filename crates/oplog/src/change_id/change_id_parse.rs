//! Reading a change ID from text: exactly 32 chars of k..z, or Malformed naming the text and what is wrong.

use super::{ChangeId, ChangeIdError};

/// What every refusal says a change ID is.
const FORM: &str = "a change ID is 32 letters k..z";

impl ChangeId {
    /// The ID this text writes, keeping the text; Malformed for any other length or a char outside k..z.
    pub fn parse(text: &str) -> Result<ChangeId, ChangeIdError> {
        let malformed = |reason: String| ChangeIdError::Malformed {
            text: text.to_string(),
            reason,
        };
        if let Some(bad) = text.chars().find(|c| !('k'..='z').contains(c)) {
            return Err(malformed(format!("{bad:?} is not a letter k..z; {FORM}")));
        }
        let length = text.chars().count();
        if length != 32 {
            return Err(malformed(format!("it has {length} chars; {FORM}")));
        }
        Ok(ChangeId(text.to_string()))
    }
}
