//! Display for Oid: the hex text.

use crate::Oid;
use std::fmt;

impl fmt::Display for Oid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
