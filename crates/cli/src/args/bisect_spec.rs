//! The verb table's entry for `mr bisect`.
//! A bare const; verbs_tests covers it.

use super::{LockNeed, VerbKind, VerbSpec};

pub const BISECT_SPEC: VerbSpec = VerbSpec {
    name: "bisect",
    kind: VerbKind::Harness,
    lock: LockNeed::Land,
    needs_config: true,
    usage: "mr bisect <last pass> <failed> --timeout <seconds> --actor <name> [--key <operation key>] \
            [--json] -- <command>...",
};
