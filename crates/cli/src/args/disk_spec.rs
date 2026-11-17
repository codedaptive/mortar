//! The verb table's entry for `mortar disk`.
//! A bare const (S-7); verbs_tests covers it.

use super::{LockNeed, VerbKind, VerbSpec};

pub const DISK_SPEC: VerbSpec = VerbSpec {
    name: "disk",
    kind: VerbKind::Harness,
    lock: LockNeed::NoLock,
    needs_config: true,
    usage: "mortar disk --actor <name> [--json]",
};
