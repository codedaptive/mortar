//! The verb table's entry for `mr uninstall` (SPEC-cli 07, "mr uninstall").
//! A bare const (S-7); verbs_setup_tests covers it.

use super::{LockNeed, VerbKind, VerbSpec};

pub const UNINSTALL_SPEC: VerbSpec = VerbSpec {
    name: "uninstall",
    kind: VerbKind::Setup,
    lock: LockNeed::NoLock,
    needs_config: false,
    usage: "mr uninstall [--json]",
};
