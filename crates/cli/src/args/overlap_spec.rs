//! The verb table's entry for `mr overlap`.
//! A bare const (S-7); verbs_tests covers it.

use super::{LockNeed, VerbKind, VerbSpec};

pub const OVERLAP_SPEC: VerbSpec = VerbSpec {
    name: "overlap",
    kind: VerbKind::Harness,
    lock: LockNeed::NoLock,
    needs_config: true,
    usage: "mr overlap <zone> --actor <name> [--json]",
};
