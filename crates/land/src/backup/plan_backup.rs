//! plan_backup: the push/delete/skip plan a backup run makes from the local and remote ref lists (SPEC-land 08).
//! plan_backup_tests cover the empty plan; the rest runs local_ref and backup_ref from other land files, so
//! tests/backup_zit covers it (S-102): a_backup_restores_every_ref_into_a_fresh_store,
//! refs_outside_the_four_prefixes_are_never_backed_up, plan_a_moved_ref_is_pushed_again_over_its_old_backup,
//! a_second_backup_pushes_nothing, a_ref_deleted_locally_is_deleted_from_the_backup,
//! plan_remote_refs_outside_this_hosts_mapped_backups_are_never_pushed_or_deleted,
//! plan_another_hosts_backup_of_the_same_ref_is_left_and_this_hosts_is_pushed and
//! plan_pushed_lists_sets_in_local_order_then_deletes_in_remote_order (plan_backup_tests' header pairs each with its
//! decision).

use crate::backup::{backup_ref, local_ref, PushPlan};
use gitio::{Oid, PushLease, PushRef};

/// `local`: the backed-up local refs as (id, full name). `remote`: every ref ls-remote listed, any name. Only the
/// remote refs local_ref maps back under `prefix` count as backups; any other is never pushed and never deleted.
/// Each local ref, in input order: no backup -> set its backup ref under lease Absent; a backup at another id ->
/// set it under a lease at that id; a backup at the same id -> unchanged, not pushed. Then each backup whose local
/// ref is gone, in remote order: deleted under a lease at its id, paired with that local name. Each lease lets the
/// push move the backup whatever its history (the backup mirrors; it is nobody's history) while refusing one that
/// moved on the remote since it was listed.
pub(crate) fn plan_backup(prefix: &str, local: &[(Oid, String)], remote: &[(Oid, String)]) -> PushPlan {
    let backups: Vec<(String, &Oid, &String)> = remote
        .iter()
        .filter_map(|(oid, name)| local_ref(prefix, name).map(|local_name| (local_name, oid, name)))
        .collect();
    let mut plan = PushPlan { push: Vec::new(), unchanged: Vec::new() };
    for (oid, name) in local {
        let lease = match backups.iter().find(|(local_name, _, _)| local_name == name) {
            None => PushLease::Absent,
            Some((_, backup, _)) if *backup == oid => {
                plan.unchanged.push(name.clone());
                continue;
            }
            Some((_, backup, _)) => PushLease::At((*backup).clone()),
        };
        let refname = backup_ref(prefix, name);
        plan.push.push((name.clone(), PushRef { refname, new: Some(oid.clone()), lease }));
    }
    for (local_name, oid, refname) in backups {
        if local.iter().all(|(_, name)| *name != local_name) {
            let delete = PushRef { refname: refname.clone(), new: None, lease: PushLease::At(oid.clone()) };
            plan.push.push((local_name, delete));
        }
    }
    plan
}
