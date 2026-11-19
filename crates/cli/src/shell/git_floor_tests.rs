//! Pure tests of the git floor check's exemptions, boundary, and refusals.

use std::cell::Cell;

use gitio::Version;

use super::{git_floor, git_floor_report};

macro_rules! exempt_verb_test {
    ($name:ident, $verb:literal) => {
        #[test]
        fn $name() {
            let calls = Cell::new(0);
            let read_version = || {
                calls.set(calls.get() + 1);
                Version { major: 2, minor: 45, patch: 2 }.require_floor()
            };

            assert!(git_floor($verb, &read_version).is_ok());
            assert_eq!(calls.get(), 0);
        }
    };
}

exempt_verb_test!(help_skips_version_read, "help");
exempt_verb_test!(install_skips_version_read, "install");
exempt_verb_test!(uninstall_skips_version_read, "uninstall");
exempt_verb_test!(guard_hook_skips_version_read, "guard-hook");

#[test]
fn below_floor_is_refused_after_one_version_read() {
    let calls = Cell::new(0);
    let read_version = || {
        calls.set(calls.get() + 1);
        Version { major: 2, minor: 45, patch: 2 }.require_floor()
    };
    let error = Version { major: 2, minor: 45, patch: 2 }.require_floor().unwrap_err();

    assert_eq!(git_floor("status", &read_version), Err(git_floor_report("status", &error)));
    assert_eq!(calls.get(), 1);
}

#[test]
fn floor_version_is_accepted() {
    let calls = Cell::new(0);
    let read_version = || {
        calls.set(calls.get() + 1);
        Version { major: 2, minor: 46, patch: 0 }.require_floor()
    };

    assert_eq!(git_floor("status", &read_version), Ok(()));
    assert_eq!(calls.get(), 1);
}

#[test]
fn unreadable_version_is_refused() {
    let calls = Cell::new(0);
    let read_version = || {
        calls.set(calls.get() + 1);
        Version::parse(b"not git\n")
    };
    let error = Version::parse(b"not git\n").unwrap_err();

    assert_eq!(git_floor("status", &read_version), Err(git_floor_report("status", &error)));
    assert_eq!(calls.get(), 1);
}
