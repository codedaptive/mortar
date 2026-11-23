//! The verb table's entry for `mr folder`.
//! A bare const; verbs_tests covers it.

use super::{LockNeed, VerbKind, VerbSpec};

pub const FOLDER_SPEC: VerbSpec = VerbSpec {
    name: "folder",
    kind: VerbKind::Harness,
    lock: LockNeed::Join,
    needs_config: true,
    usage: "mr folder (zone <zone> <path> [--start <commit>] | request <zone> <request> <path> --paths \
            <path>... [--with <zone>]... | remove <path> | attempts <n> <dir> | keep <k> | drop | refresh <zone folder \
            path>) --actor <name> [--key <operation key>] [--json]",
};
