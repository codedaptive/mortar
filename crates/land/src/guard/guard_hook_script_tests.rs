//! guard_hook_script: each hook's exact script, as the contract writes it.

use super::guard_hook_script::guard_hook_script;

#[test]
fn reference_transaction_script() {
    assert_eq!(
        guard_hook_script("reference-transaction"),
        "#!/bin/sh\nexec mortar guard-hook reference-transaction \"$@\"\n"
    );
}

#[test]
fn pre_push_script() {
    assert_eq!(guard_hook_script("pre-push"), "#!/bin/sh\nexec mortar guard-hook pre-push \"$@\"\n");
}
