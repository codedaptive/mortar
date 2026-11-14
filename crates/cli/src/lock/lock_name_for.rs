//! The oplog lock a verb takes before it moves a ref (S-16).
//! Shell API with no caller yet: the ref-moving verbs of WO-cli-2..6 call it.

use oplog::LockName;

use crate::args::LockNeed;

/// Join is `zone`'s join lock, Land the landing lock, Folders the folders lock; NoLock takes none (None).
pub fn lock_name_for(need: LockNeed, zone: &str) -> Option<LockName> {
    match need {
        LockNeed::NoLock => None,
        LockNeed::Join => Some(LockName::Join(zone.to_string())),
        LockNeed::Land => Some(LockName::Land),
        LockNeed::Folders => Some(LockName::Folders),
    }
}
