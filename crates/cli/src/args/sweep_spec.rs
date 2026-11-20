//! The verb table's entry for `mr sweep`.
//! A bare const; verbs_tests covers it.

use super::{LockNeed, VerbKind, VerbSpec};

pub const SWEEP_SPEC: VerbSpec = VerbSpec {
    name: "sweep",
    kind: VerbKind::Harness,
    lock: LockNeed::Folders,
    needs_config: true,
    usage: "mr sweep [--idle-days <n>] [--ref <full ref name>]... --actor <name> [--key <operation key>] [--json]",
};
