//! How a change ID prints: its 32 chars, as the Mortar-Change-Id trailer carries it.

use std::fmt;

use super::ChangeId;

impl fmt::Display for ChangeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
