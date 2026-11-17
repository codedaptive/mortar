//! classify_hook: no file is Missing. Mortar and Foreign are decided against guard_hook_script, another land file
//! (S-102), so their tests go through guard_status and guard_install. No such test is in land today: they are owed
//! to Nagatha as RITs. Mortar, the exact script: cases 1 and 2 in
//! ledger/reports/WO-land-guard-symref-for-nagatha.md. Foreign: a user's hook (cases 3, 4 and 7), the
//! other hook's script (case 9), the script with one byte more (case 10) and an empty file (case 11).

use super::classify_hook::classify_hook;
use super::HookState;

#[test]
fn no_file_is_missing() {
    assert_eq!(classify_hook("pre-push", None), HookState::Missing);
}
