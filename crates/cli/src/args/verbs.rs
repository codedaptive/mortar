//! The verbs the binary knows: only those that are built.

use super::{LockNeed, VerbKind, VerbSpec};

/// In the order the usage error lists them.
pub const VERBS: &[VerbSpec] = &[VerbSpec {
    name: "init",
    kind: VerbKind::Harness,
    lock: LockNeed::NoLock,
    needs_config: false,
    usage: "mortar init --actor <name> --origin <branch> --integration <branch> --remote <name> \
            [--zone <zone>=<path>[,<path>...]]... [--root-file <path>]... [--build-dir <path>]... \
            [--cap-bytes <n>] [--floor-bytes <n>] [--formatter <word>]... [--formatter-timeout <seconds>] \
            [--replace] [--json]",
}];
