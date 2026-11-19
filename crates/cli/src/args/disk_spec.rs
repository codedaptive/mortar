//! The verb table's entry for `mr disk`.
//! A bare const; verbs_tests covers it.

use super::{LockNeed, VerbKind, VerbSpec};

pub const DISK_SPEC: VerbSpec = VerbSpec {
    name: "disk",
    kind: VerbKind::Harness,
    lock: LockNeed::NoLock,
    needs_config: true,
    usage: "mr disk --actor <name> [--json]",
};
