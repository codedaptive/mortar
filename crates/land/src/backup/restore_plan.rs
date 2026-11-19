//! restore_plan: each backup ref on the remote against the local store, before anything is fetched.
//! No sibling unit test: every test of it runs local_ref from another land file. Covered by
//! tests/backup_zit through restore_backup: a ref absent locally is Created
//! (a_backup_restores_every_ref_into_a_fresh_store); one at the backup's id is Same and one at another id Differs
//! (restore_lists_a_differing_ref_without_moving_it_and_an_equal_one_as_same); a backed-up heads/main gives no entry
//! (plan_restore_of_a_backed_up_main_gives_nothing_and_creates_nothing); HEAD, heads/*, another host's backup and a
//! backed-up tag give none (plan_restore_skips_refs_outside_this_hosts_backup); entries keep the remote's order
//! (plan_restore_entries_come_in_remote_order).

use crate::backup::{local_ref, Restored};
use gitio::Oid;

/// `remote` is every ref ls-remote listed, `local` the local refs under BACKUP_PREFIXES, both as (id, refname).
/// Only remote names that local_ref maps back under `prefix` count, in remote order; each comes out as its backup
/// ref name with what restore is to do: local ref absent, Created at the backup's id (to be created; restore_backup
/// turns it into Vanished when the fetch finds it gone); local ref at the backup's id, Same; at another id, Differs.
pub(crate) fn restore_plan(prefix: &str, remote: &[(Oid, String)], local: &[(Oid, String)]) -> Vec<(String, Restored)> {
    remote
        .iter()
        .filter_map(|(backup, name)| {
            let refname = local_ref(prefix, name)?;
            let restored = match local.iter().find(|(_, here)| *here == refname) {
                None => Restored::Created { refname, oid: backup.clone() },
                Some((here, _)) if here == backup => Restored::Same { refname },
                Some((here, _)) => Restored::Differs { refname, local: here.clone(), backup: backup.clone() },
            };
            Some((name.clone(), restored))
        })
        .collect()
}
