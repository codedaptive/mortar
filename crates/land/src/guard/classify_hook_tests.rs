//! classify_hook: no file is Missing. Mortar and Foreign are decided against guard_hook_script, another land file
//! (S-102), so tests/guard_zit proves them through guard_status and guard_install. Mortar, the exact script:
//! fresh_install_writes_config_and_hooks, repeat_install_changes_nothing. Foreign: a user's hook
//! (status_of_an_untouched_repo_and_of_a_foreign_hook, a_foreign_pre_push_is_refused_and_nothing_written,
//! a_foreign_reference_transaction_is_refused_and_nothing_written), the other hook's script
//! (foreign_pre_push_holding_the_reference_transaction_script), the script with one byte more
//! (foreign_pre_push_of_the_exact_script_plus_one_byte) and an empty file (foreign_pre_push_that_is_empty).

use super::classify_hook::classify_hook;
use super::HookState;

#[test]
fn no_file_is_missing() {
    assert_eq!(classify_hook("pre-push", None), HookState::Missing);
}
