//! The verbs the binary knows: only those that are built.

use super::{LockNeed, VerbKind, VerbSpec};

/// In the order the usage error lists them.
pub const VERBS: &[VerbSpec] = &[
    VerbSpec {
        name: "guard-hook",
        kind: VerbKind::Hook,
        lock: LockNeed::NoLock,
        needs_config: false,
        usage: "mortar guard-hook (reference-transaction <state> | pre-push <remote> <url>)",
    },
    VerbSpec {
        name: "init",
        kind: VerbKind::Harness,
        lock: LockNeed::NoLock,
        needs_config: false,
        usage: "mortar init --actor <name> --origin <branch> --integration <branch> --remote <name> \
                [--zone <zone>=<path>[,<path>...]]... [--root-file <path>]... [--build-dir <path>]... \
                [--cap-bytes <n>] [--floor-bytes <n>] [--formatter <word>]... [--formatter-timeout <seconds>] \
                [--replace] [--json]",
    },
    VerbSpec {
        name: "join",
        kind: VerbKind::Agent,
        lock: LockNeed::Join,
        needs_config: true,
        usage: "mortar join [--key <operation key>] [--json]",
    },
    VerbSpec {
        name: "log",
        kind: VerbKind::ReadOnly,
        lock: LockNeed::NoLock,
        needs_config: true,
        usage: "mortar log [--ref <full ref name>] [--actor <name>] [--key <operation key>] [--json]",
    },
    VerbSpec {
        name: "need",
        kind: VerbKind::Agent,
        lock: LockNeed::Join,
        needs_config: true,
        usage: "mortar need <path> [--key <key>] [--json]",
    },
    VerbSpec {
        name: "resolve",
        kind: VerbKind::Agent,
        lock: LockNeed::Join,
        needs_config: true,
        usage: "mortar resolve [--key <operation key>] [--json]",
    },
    VerbSpec {
        name: "restore",
        kind: VerbKind::Harness,
        lock: LockNeed::FromRefs,
        needs_config: true,
        usage: "mortar restore <op id> <full ref name>... --actor <name> [--key <operation key>] [--json]",
    },
    VerbSpec {
        name: "save",
        kind: VerbKind::Agent,
        lock: LockNeed::Join,
        needs_config: true,
        usage: "mortar save \"<sentence>\" [--key <key>] [--json]",
    },
    VerbSpec {
        name: "status",
        kind: VerbKind::Agent,
        lock: LockNeed::NoLock,
        needs_config: true,
        usage: "mortar status [--json]",
    },
];
