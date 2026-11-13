//! Refusal texts: each test checks its variant names the holder (pid, host, key, since), path, zone or
//! reason it carries, and the action it tells the reader to take.

use std::path::PathBuf;

use super::{LockError, LockRecord};

fn holder() -> LockRecord {
    LockRecord {
        pid: 4242,
        host: "build-mac".to_string(),
        key: "join:zone-a".to_string(),
        since: 1763030400,
    }
}

fn assert_names_holder(text: &str, path: &str) {
    for part in ["4242", "build-mac", "join:zone-a", "1763030400", path] {
        assert!(text.contains(part), "{text:?} lacks {part:?}");
    }
}

#[test]
fn held_names_holder_path_and_says_wait() {
    let err = LockError::Held {
        path: PathBuf::from("/repo/.git/mortar/locks/land.lock"),
        holder: holder(),
    };
    let text = err.to_string();
    assert_names_holder(&text, "/repo/.git/mortar/locks/land.lock");
    assert!(text.contains("wait for it to finish"), "{text}");
}

#[test]
fn takeover_in_progress_names_guard_holder_and_removal() {
    let err = LockError::TakeoverInProgress {
        path: PathBuf::from("/repo/.git/mortar/locks/land.lock.takeover"),
        holder: holder(),
    };
    let text = err.to_string();
    assert_names_holder(&text, "/repo/.git/mortar/locks/land.lock.takeover");
    assert!(text.contains("running or crashed"), "{text}");
    assert!(text.contains("by hand"), "{text}");
}

#[test]
fn not_ours_with_holder_names_holder() {
    let err = LockError::NotOurs {
        path: PathBuf::from("/repo/.git/mortar/locks/join-zone-a.lock"),
        holder: Some(holder()),
    };
    let text = err.to_string();
    assert_names_holder(&text, "/repo/.git/mortar/locks/join-zone-a.lock");
    assert!(text.contains("leave it, its holder releases it"), "{text}");
}

#[test]
fn not_ours_without_file_says_no_lock_file() {
    let err = LockError::NotOurs {
        path: PathBuf::from("/repo/.git/mortar/locks/join-zone-a.lock"),
        holder: None,
    };
    let text = err.to_string();
    assert!(text.contains("/repo/.git/mortar/locks/join-zone-a.lock"), "{text}");
    assert!(text.contains("no lock file, so nothing to release"), "{text}");
}

#[test]
fn contended_names_path_and_says_try_again() {
    let err = LockError::Contended {
        path: PathBuf::from("/repo/.git/mortar/locks/land.lock"),
    };
    assert_eq!(
        err.to_string(),
        "/repo/.git/mortar/locks/land.lock changed hands repeatedly while acquiring; try again"
    );
}

#[test]
fn bad_name_names_the_zone() {
    let text = LockError::BadName("a/b".to_string()).to_string();
    assert!(text.contains("\"a/b\""), "{text}");
    assert!(text.contains("use 1 to 100 ASCII letters"), "{text}");
}

#[test]
fn bad_record_names_the_reason() {
    let text = LockError::BadRecord("host is empty".to_string()).to_string();
    assert!(text.contains("host is empty"), "{text}");
    assert!(text.contains("fix the host or op key given to the lock"), "{text}");
}

#[test]
fn malformed_names_path_and_reason() {
    let err = LockError::Malformed {
        path: Some(PathBuf::from("/repo/.git/mortar/locks/land.lock")),
        reason: "has 3 lines, not 4".to_string(),
    };
    let text = err.to_string();
    assert!(text.contains("/repo/.git/mortar/locks/land.lock"), "{text}");
    assert!(text.contains("has 3 lines, not 4"), "{text}");
    assert!(text.contains("then remove it by hand"), "{text}");
}

#[test]
fn malformed_without_path_names_reason() {
    let err = LockError::Malformed {
        path: None,
        reason: "pid is empty".to_string(),
    };
    let text = err.to_string();
    assert!(text.contains("pid is empty"), "{text}");
    assert!(text.contains("mortar did not write this text, so check what did"), "{text}");
}

#[test]
fn host_unknown_names_the_reason() {
    let text = LockError::HostUnknown("hostname printed nothing".to_string()).to_string();
    assert!(text.contains("hostname printed nothing"), "{text}");
    assert!(text.contains("check that `hostname` runs on this machine"), "{text}");
}

#[test]
fn io_names_path_and_cause() {
    let err = LockError::Io {
        path: PathBuf::from("/repo/.git/mortar/locks"),
        source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied here"),
    };
    let text = err.to_string();
    assert!(text.contains("/repo/.git/mortar/locks"), "{text}");
    assert!(text.contains("denied here"), "{text}");
    assert!(text.contains("check that path's folder and its permissions, then run again"), "{text}");
}
