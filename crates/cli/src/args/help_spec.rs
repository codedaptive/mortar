//! The verb table's entry for `mr help` (SPEC-cli 08). entry::main_entry answers help before `run` parses the line,
//! so this entry never reaches dispatch; it is in the table so a usage error lists help and `mr help help` has a
//! usage.
//! A bare const (S-7); verbs_setup_tests covers it.

use super::{LockNeed, VerbKind, VerbSpec};

pub const HELP_SPEC: VerbSpec = VerbSpec {
    name: "help",
    kind: VerbKind::Setup,
    lock: LockNeed::NoLock,
    needs_config: false,
    usage: "mr help [<verb>] [--json]",
};
