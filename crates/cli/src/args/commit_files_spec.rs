//! The verb table's entry for `mr commit-files`.
//! A bare const; verbs_tests covers it.

use super::{LockNeed, VerbKind, VerbSpec};

pub const COMMIT_FILES_SPEC: VerbSpec = VerbSpec {
    name: "commit-files",
    kind: VerbKind::Harness,
    lock: LockNeed::FromFolder,
    needs_config: true,
    usage: "mr commit-files <path>... [--message <text>] --actor <name> [--key <operation key>] [--json]",
};
