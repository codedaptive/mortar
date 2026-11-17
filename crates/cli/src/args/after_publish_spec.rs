//! The verb table's entry for `mortar after-publish`.
//! A bare const (S-7); verbs_tests covers it.

use super::{LockNeed, VerbKind, VerbSpec};

pub const AFTER_PUBLISH_SPEC: VerbSpec = VerbSpec {
    name: "after-publish",
    kind: VerbKind::Harness,
    lock: LockNeed::Land,
    needs_config: true,
    usage: "mortar after-publish <zone> [--move <zone>]... --actor <name> [--key <operation key>] [--json]",
};
