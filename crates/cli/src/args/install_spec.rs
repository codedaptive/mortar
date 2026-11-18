//! The verb table's entry for `mr install` (SPEC-cli 07, "mr install").
//! A bare const (S-7); verbs_setup_tests covers it.

use super::{LockNeed, VerbKind, VerbSpec};

pub const INSTALL_SPEC: VerbSpec = VerbSpec {
    name: "install",
    kind: VerbKind::Setup,
    lock: LockNeed::NoLock,
    needs_config: false,
    usage: "mr install [--no-place] [--short-name] [--target <harness>[,<harness>...]] [--mode skill] \
            [--location global|local] [-y] [--json]",
};
