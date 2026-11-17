//! The verb table's entry for `mortar restore-backup`.
//! A bare const (S-7); verbs_tests covers it.

use super::{LockNeed, VerbKind, VerbSpec};

pub const RESTORE_BACKUP_SPEC: VerbSpec = VerbSpec {
    name: "restore-backup",
    kind: VerbKind::Harness,
    lock: LockNeed::Land,
    needs_config: true,
    usage: "mortar restore-backup <host> --actor <name> [--key <operation key>] [--json]",
};
