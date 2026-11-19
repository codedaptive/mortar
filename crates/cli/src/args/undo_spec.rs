//! The verb table's entry for `mr undo`.
//! A bare const; verbs_tests covers it.

use super::{LockNeed, VerbKind, VerbSpec};

pub const UNDO_SPEC: VerbSpec = VerbSpec {
    name: "undo",
    kind: VerbKind::Agent,
    lock: LockNeed::Join,
    needs_config: true,
    usage: "mr undo [--key <key>] [--json]",
};
