//! classify_hook: an absent file is Missing. Mortar and Foreign are decided against guard_hook_script, another land
//! file (S-102), so their tests go through guard_status and guard_install, in tests/land_rit/guard_installation.rs.
//! Mortar: the exact script. Foreign: a user's hook, the other hook's script, the script with one byte more and an
//! empty file.

use super::classify_hook::classify_hook;
use super::HookState;

#[test]
fn no_file_is_missing() {
    assert_eq!(classify_hook("pre-push", None), HookState::Missing);
}
