//! Which oplog lock a verb takes before it moves a ref (S-16).
//! A bare type (S-7); verbs_tests exercises it.

/// Join is the zone's `join-<zone>` lock; Land the `land` lock; Folders the `folders` lock. FromRefs: the verb
/// chooses its lock on each call from the refs it names (`mr restore`, restore_lock, S-113(1)).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockNeed {
    NoLock,
    Join,
    Land,
    Folders,
    FromRefs,
}
