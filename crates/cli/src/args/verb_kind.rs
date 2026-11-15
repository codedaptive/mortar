//! Who runs a verb.
//! A bare type (S-7); verbs_tests exercises it.

/// Agent: the agent verbs of the feature list's §1, which take their folder and actor from the folder record.
/// Harness: the verbs a harness runs, which take `--actor`.
/// Hook: run by a git hook Mortar installed (`mortar guard-hook`): reads no folder, takes no `--actor`, acts as "git".
/// ReadOnly: run by an operator or a harness, reads only: no lock, no op, no actor of its own; its `--actor` and
/// `--key` are filters, not who it acts as (S-114).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerbKind {
    Agent,
    Harness,
    Hook,
    ReadOnly,
}
