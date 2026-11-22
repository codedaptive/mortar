//! The verb table's entry for `mr withdraw`.
//! A bare const; verbs_tests covers it.

use super::{LockNeed, VerbKind, VerbSpec};

pub const WITHDRAW_SPEC: VerbSpec = VerbSpec {
    name: "withdraw",
    kind: VerbKind::Harness,
    lock: LockNeed::Land,
    needs_config: true,
    usage: "mr withdraw <zone> --actor <name> [--key <operation key>] [--json]",
};
