//! One verb of the binary.
//! A bare type (S-7); verbs_tests and find_verb_tests exercise it.

use super::{LockNeed, VerbKind};

/// `usage` is the verb's whole usage line, shown as the next step of its usage error. `needs_config`: the
/// verb refuses without `<common>/mortar/config.json`.
#[derive(Debug, PartialEq, Eq)]
pub struct VerbSpec {
    pub name: &'static str,
    pub kind: VerbKind,
    pub lock: LockNeed,
    pub needs_config: bool,
    pub usage: &'static str,
}
