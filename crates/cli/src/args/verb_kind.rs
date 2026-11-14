//! Who runs a verb.
//! A bare type (S-7); verbs_tests exercises it.

/// Agent: the six verbs of the feature list's §1, which take their folder and actor from the folder record.
/// Harness: the verbs a harness runs, which take `--actor`.
/// Hook: run by a git hook Mortar installed (`mortar guard-hook`): reads no folder, takes no `--actor`, acts as "git".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerbKind {
    Agent,
    Harness,
    Hook,
}
