//! Which oplog lock a verb takes before it moves a ref.
//! A bare type; verbs_tests exercises it.

/// Join is the zone's `join-<zone>` lock; Land the `land` lock; Folders the `folders` lock. FromRefs: the verb
/// chooses its lock on each call from the refs it names (`mr restore`, restore_lock). FromFolder: the verb chooses
/// its lock on each call from the folder it runs in: `mr commit-files` takes land in the integration folder and
/// join-<zone> in a zone folder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockNeed {
    NoLock,
    Join,
    Land,
    Folders,
    FromRefs,
    FromFolder,
}
