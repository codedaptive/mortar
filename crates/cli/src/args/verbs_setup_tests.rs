//! VERBS: install, uninstall and help are setup verbs: no lock, no config, and their usage lines.
//! ZUT (rule 11): the verb table is pure data.

use super::{LockNeed, VerbKind, VerbSpec, VERBS};

fn verb(name: &str) -> &'static VerbSpec {
    VERBS.iter().find(|spec| spec.name == name).expect("the verb is in the table")
}

#[test]
fn install_is_a_setup_verb_without_lock_or_config() {
    let install = verb("install");
    assert_eq!(install.kind, VerbKind::Setup);
    assert_eq!(install.lock, LockNeed::NoLock);
    assert!(!install.needs_config);
    assert_eq!(
        install.usage,
        "mr install [--no-place] [--short-name] [--target <harness>[,<harness>...]] [--mode skill] \
         [--location global|local] [-y] [--json]"
    );
}

#[test]
fn uninstall_is_a_setup_verb_without_lock_or_config() {
    let uninstall = verb("uninstall");
    assert_eq!(uninstall.kind, VerbKind::Setup);
    assert_eq!(uninstall.lock, LockNeed::NoLock);
    assert!(!uninstall.needs_config);
    assert_eq!(uninstall.usage, "mr uninstall [--json]");
}

#[test]
fn help_is_a_setup_verb_without_lock_or_config() {
    let help = verb("help");
    assert_eq!(help.kind, VerbKind::Setup);
    assert_eq!(help.lock, LockNeed::NoLock);
    assert!(!help.needs_config);
    assert_eq!(help.usage, "mr help [<verb>] [--json]");
}
