//! lock_name_for: each need to its oplog lock, the join lock carrying the zone; none for the needs whose verb
//! chooses its lock (FromRefs, FromFolder).

use oplog::LockName;

use super::lock_name_for;
use crate::args::LockNeed;

#[test]
fn no_lock_takes_none() {
    assert_eq!(lock_name_for(LockNeed::NoLock, "a"), None);
}

#[test]
fn join_takes_the_zones_join_lock() {
    assert_eq!(lock_name_for(LockNeed::Join, "a"), Some(LockName::Join("a".to_string())));
}

#[test]
fn land_and_folders_take_their_own_locks_whatever_the_zone() {
    assert_eq!(lock_name_for(LockNeed::Land, "a"), Some(LockName::Land));
    assert_eq!(lock_name_for(LockNeed::Folders, "a"), Some(LockName::Folders));
}

#[test]
fn from_refs_and_from_folder_take_none_here_because_the_verb_chooses_its_lock() {
    for need in [LockNeed::FromRefs, LockNeed::FromFolder] {
        assert_eq!(lock_name_for(need, "a"), None, "{need:?}");
    }
}
