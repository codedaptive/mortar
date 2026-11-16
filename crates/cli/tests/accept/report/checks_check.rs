//! Records one check line.

use super::{Checks, Line};

impl Checks {
    /// Records one check: what was done, whether it held, what was seen. Returns `ok`, so a body can stop at a
    /// check the later ones depend on.
    pub fn check(&mut self, did: &str, ok: bool, saw: &str) -> bool {
        self.lines.push(Line::Check { ok, did: did.to_string(), saw: saw.to_string() });
        ok
    }
}
