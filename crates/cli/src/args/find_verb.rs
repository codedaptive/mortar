//! A verb by its name.

use super::{VerbSpec, VERBS};

/// The entry of VERBS named exactly `name`; None for any other word.
pub fn find_verb(name: &str) -> Option<&'static VerbSpec> {
    VERBS.iter().find(|verb| verb.name == name)
}
