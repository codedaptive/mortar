//! The verb table's entry for `mr snapshot`.
//! A bare const (S-7); verbs_tests covers it.

use super::{LockNeed, VerbKind, VerbSpec};

pub const SNAPSHOT_SPEC: VerbSpec = VerbSpec {
    name: "snapshot",
    kind: VerbKind::Harness,
    lock: LockNeed::Join,
    needs_config: true,
    usage: "mr snapshot --actor <name> [--key <operation key>] [--json]",
};
