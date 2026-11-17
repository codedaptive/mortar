//! The oplog lock a verb takes before it moves a ref (S-16).
//! Shell API: `mortar join` and `mortar resolve` (through request_run_for), `mortar need` (run_need) and
//! `mortar save` (run_save) call it; the other verbs that move a ref name their lock themselves.

use oplog::LockName;

use crate::args::LockNeed;

/// Join is `zone`'s join lock, Land the landing lock, Folders the folders lock; NoLock takes none (None), and so does
/// FromRefs: that verb chooses its lock from the refs it names (restore_lock).
pub fn lock_name_for(need: LockNeed, zone: &str) -> Option<LockName> {
    match need {
        LockNeed::NoLock => None,
        LockNeed::Join => Some(LockName::Join(zone.to_string())),
        LockNeed::Land => Some(LockName::Land),
        LockNeed::Folders => Some(LockName::Folders),
        LockNeed::FromRefs => None,
    }
}
