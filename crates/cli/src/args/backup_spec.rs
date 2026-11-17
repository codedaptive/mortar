//! The verb table's entry for `mr backup`.
//! A bare const (S-7); verbs_tests covers it.

use super::{LockNeed, VerbKind, VerbSpec};

pub const BACKUP_SPEC: VerbSpec = VerbSpec {
    name: "backup",
    kind: VerbKind::Harness,
    lock: LockNeed::Land,
    needs_config: true,
    usage: "mr backup --actor <name> [--key <operation key>] [--json]",
};
