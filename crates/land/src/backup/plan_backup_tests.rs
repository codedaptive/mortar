//! plan_backup: with no local refs and no remote refs the plan is empty. Every other decision runs local_ref or
//! backup_ref from another land file (S-102), so tests/backup_zit proves it through backup: no backup is set under
//! lease Absent (a_backup_restores_every_ref_into_a_fresh_store, refs_outside_the_four_prefixes_are_never_backed_up);
//! a backup at another id is set under a lease at that id (plan_a_moved_ref_is_pushed_again_over_its_old_backup); one
//! at the same id is unchanged (a_second_backup_pushes_nothing); one whose local ref is gone is deleted
//! (a_ref_deleted_locally_is_deleted_from_the_backup); remote refs that map to no backed-up local ref are never
//! pushed or deleted (plan_remote_refs_outside_this_hosts_mapped_backups_are_never_pushed_or_deleted); another
//! host's backup is not this host's (plan_another_hosts_backup_of_the_same_ref_is_left_and_this_hosts_is_pushed);
//! sets come in local order, then deletes in remote order
//! (plan_pushed_lists_sets_in_local_order_then_deletes_in_remote_order).

use super::{plan_backup, PushPlan};

const PREFIX: &str = "refs/mortar-backup/h1/";

#[test]
fn nothing_local_and_no_backup_plans_nothing() {
    assert_eq!(plan_backup(PREFIX, &[], &[]), PushPlan { push: vec![], unchanged: vec![] });
}
