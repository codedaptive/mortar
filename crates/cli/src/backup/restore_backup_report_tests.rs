//! restore_backup_report: a host with no backup, one line per restored ref in order for each Restored arm, and a
//! next step that names the refs left as they are only when one differs.

use gitio::Oid;
use land::Restored;

use super::restore_backup_report;
use crate::report::{ExitCode, OutValue, Report};

const A: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";
const B: &str = "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391";

const DIFFERS_NEXT: &str = "each ref left as it is holds another commit than its backup under \
                            refs/mortar-backup/build-mac-02/ on the remote; keep it, or move it to the backup's commit \
                            by hand";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn texts(name: &str, words: &[&str]) -> (String, OutValue) {
    (name.to_string(), OutValue::Texts(words.iter().map(ToString::to_string).collect()))
}

fn restored(lines: &[&str], next: &str, fields: [&[&str]; 4]) -> Report {
    Report {
        verb: "restore-backup".to_string(),
        exit: ExitCode::Done,
        outcome: "restored".to_string(),
        lines: lines.iter().map(ToString::to_string).collect(),
        owner: None,
        next: next.to_string(),
        fields: vec![
            texts("created", fields[0]),
            texts("same", fields[1]),
            texts("differs", fields[2]),
            texts("vanished", fields[3]),
        ],
    }
}

#[test]
fn no_entries_says_the_host_has_no_backup() {
    assert_eq!(
        restore_backup_report("build-mac-02", &[]),
        restored(&["host build-mac-02 has no backup on the remote"], "nothing to do", [&[], &[], &[], &[]])
    );
}

#[test]
fn created_names_the_ref_and_its_commit() {
    let entries = [Restored::Created { refname: "refs/heads/zone/cli".to_string(), oid: oid(A) }];
    assert_eq!(
        restore_backup_report("build-mac-02", &entries),
        restored(
            &["created refs/heads/zone/cli at 4b825dc642cb6eb9a060e54bf8d69288fbee4904"],
            "nothing to do",
            [&["refs/heads/zone/cli"], &[], &[], &[]],
        )
    );
}

#[test]
fn same_says_the_ref_already_holds_the_backups_commit() {
    let entries = [Restored::Same { refname: "refs/mortar/squash/cli".to_string() }];
    assert_eq!(
        restore_backup_report("build-mac-02", &entries),
        restored(
            &["refs/mortar/squash/cli already holds the backup's commit"],
            "nothing to do",
            [&[], &["refs/mortar/squash/cli"], &[], &[]],
        )
    );
}

#[test]
fn differs_names_both_commits_and_the_next_step_names_the_hosts_backup() {
    let entries = [Restored::Differs { refname: "refs/heads/zone/land".to_string(), local: oid(A), backup: oid(B) }];
    assert_eq!(
        restore_backup_report("build-mac-02", &entries),
        restored(
            &["refs/heads/zone/land is at 4b825dc642cb6eb9a060e54bf8d69288fbee4904, its backup at \
               e69de29bb2d1d6434b8b29ae775ad8c2e48c5391: left as it is"],
            DIFFERS_NEXT,
            [&[], &[], &["refs/heads/zone/land"], &[]],
        )
    );
}

#[test]
fn vanished_says_the_ref_was_not_created() {
    let entries = [Restored::Vanished { refname: "refs/heads/req/cli/WO-9".to_string() }];
    assert_eq!(
        restore_backup_report("build-mac-02", &entries),
        restored(
            &["refs/heads/req/cli/WO-9 was gone from the remote by the fetch: not created"],
            "nothing to do",
            [&[], &[], &[], &["refs/heads/req/cli/WO-9"]],
        )
    );
}

#[test]
fn every_entry_is_a_line_in_order_and_one_differs_sets_the_next_step() {
    let entries = [
        Restored::Vanished { refname: "refs/heads/req/cli/WO-9".to_string() },
        Restored::Created { refname: "refs/heads/zone/cli".to_string(), oid: oid(A) },
        Restored::Differs { refname: "refs/heads/zone/land".to_string(), local: oid(B), backup: oid(A) },
        Restored::Same { refname: "refs/mortar/squash/cli".to_string() },
        Restored::Created { refname: "refs/heads/zone/join".to_string(), oid: oid(B) },
    ];
    assert_eq!(
        restore_backup_report("build-mac-02", &entries),
        restored(
            &[
                "refs/heads/req/cli/WO-9 was gone from the remote by the fetch: not created",
                "created refs/heads/zone/cli at 4b825dc642cb6eb9a060e54bf8d69288fbee4904",
                "refs/heads/zone/land is at e69de29bb2d1d6434b8b29ae775ad8c2e48c5391, its backup at \
                 4b825dc642cb6eb9a060e54bf8d69288fbee4904: left as it is",
                "refs/mortar/squash/cli already holds the backup's commit",
                "created refs/heads/zone/join at e69de29bb2d1d6434b8b29ae775ad8c2e48c5391",
            ],
            DIFFERS_NEXT,
            [
                &["refs/heads/zone/cli", "refs/heads/zone/join"],
                &["refs/mortar/squash/cli"],
                &["refs/heads/zone/land"],
                &["refs/heads/req/cli/WO-9"],
            ],
        )
    );
}

#[test]
fn no_differs_among_several_entries_is_nothing_to_do() {
    let entries = [
        Restored::Created { refname: "refs/heads/zone/cli".to_string(), oid: oid(A) },
        Restored::Same { refname: "refs/mortar/squash/cli".to_string() },
        Restored::Vanished { refname: "refs/heads/req/cli/WO-9".to_string() },
    ];
    assert_eq!(restore_backup_report("build-mac-02", &entries).next, "nothing to do");
}

#[test]
fn the_differs_next_step_names_the_host_given() {
    let entries = [Restored::Differs { refname: "refs/heads/zone/a".to_string(), local: oid(A), backup: oid(B) }];
    assert_eq!(
        restore_backup_report("h1", &entries).next,
        "each ref left as it is holds another commit than its backup under refs/mortar-backup/h1/ on the remote; \
         keep it, or move it to the backup's commit by hand"
    );
}
