//! The verb table's entry for `mortar undo`.
//! A bare const (S-7); verbs_tests covers it.

use super::{LockNeed, VerbKind, VerbSpec};

pub const UNDO_SPEC: VerbSpec = VerbSpec {
    name: "undo",
    kind: VerbKind::Agent,
    lock: LockNeed::Join,
    needs_config: true,
    usage: "mortar undo [--key <key>] [--json]",
};
