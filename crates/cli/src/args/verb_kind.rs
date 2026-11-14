//! Who runs a verb.
//! A bare type (S-7); verbs_tests exercises it.

/// Agent: the six verbs of the feature list's §1, which take their folder and actor from the folder record.
/// Harness: every other verb, which takes `--actor`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerbKind {
    Agent,
    Harness,
}
