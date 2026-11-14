//! backup_prefix: one host's namespace under refs/mortar-backup/, and the host names refused.

use super::{backup_prefix, BackupError};

#[test]
fn names_the_host_under_refs_mortar_backup() {
    assert_eq!(backup_prefix("h1").ok(), Some("refs/mortar-backup/h1/".to_string()));
}

#[test]
fn keeps_a_dotted_host_name_whole() {
    assert_eq!(backup_prefix("box.local").ok(), Some("refs/mortar-backup/box.local/".to_string()));
}

#[test]
fn an_empty_host_is_bad_host() {
    match backup_prefix("") {
        Err(BackupError::BadHost(host)) => assert_eq!(host, ""),
        other => panic!("expected BadHost, got {other:?}"),
    }
}

#[test]
fn a_host_with_a_slash_is_bad_host() {
    match backup_prefix("a/b") {
        Err(BackupError::BadHost(host)) => assert_eq!(host, "a/b"),
        other => panic!("expected BadHost, got {other:?}"),
    }
}

#[test]
fn a_host_that_is_only_a_slash_is_bad_host() {
    match backup_prefix("/") {
        Err(BackupError::BadHost(host)) => assert_eq!(host, "/"),
        other => panic!("expected BadHost, got {other:?}"),
    }
}
