//! backup_report: what one backup pushed, or that nothing needed updating, and how many refs were already backed up.

use land::Backup;

use super::backup_report;
use crate::report::{ExitCode, OutValue, Report};

fn strings(words: &[&str]) -> Vec<String> {
    words.iter().map(ToString::to_string).collect()
}

fn backed_up(lines: &[&str], pushed: &[&str], unchanged: &[&str]) -> Report {
    Report {
        verb: "backup".to_string(),
        exit: ExitCode::Done,
        outcome: "backed-up".to_string(),
        lines: strings(lines),
        owner: None,
        next: "nothing to do".to_string(),
        fields: vec![
            ("pushed".to_string(), OutValue::Texts(strings(pushed))),
            ("unchanged".to_string(), OutValue::Texts(strings(unchanged))),
        ],
    }
}

#[test]
fn nothing_pushed_says_the_backup_already_holds_every_ref() {
    let answer = Backup {
        pushed: Vec::new(),
        unchanged: strings(&["refs/heads/zone/cli", "refs/mortar/squash/cli"]),
    };
    assert_eq!(
        backup_report(&answer),
        backed_up(
            &[
                "nothing to update: this host's backup on the remote already holds every in-flight ref",
                "2 refs were already backed up",
            ],
            &[],
            &["refs/heads/zone/cli", "refs/mortar/squash/cli"],
        )
    );
}

#[test]
fn pushed_refs_are_counted_then_listed_one_per_line() {
    let answer = Backup {
        pushed: strings(&["refs/heads/zone/land", "refs/heads/req/land/WO-7"]),
        unchanged: strings(&["refs/heads/zone/cli", "refs/mortar/squash/cli", "refs/heads/zone/join"]),
    };
    assert_eq!(
        backup_report(&answer),
        backed_up(
            &[
                "updated 2 refs in this host's backup on the remote:",
                "refs/heads/zone/land",
                "refs/heads/req/land/WO-7",
                "3 refs were already backed up",
            ],
            &["refs/heads/zone/land", "refs/heads/req/land/WO-7"],
            &["refs/heads/zone/cli", "refs/mortar/squash/cli", "refs/heads/zone/join"],
        )
    );
}

#[test]
fn pushed_with_nothing_unchanged_says_zero_were_already_backed_up() {
    let answer = Backup { pushed: strings(&["refs/heads/zone/land"]), unchanged: Vec::new() };
    assert_eq!(
        backup_report(&answer),
        backed_up(
            &[
                "updated 1 refs in this host's backup on the remote:",
                "refs/heads/zone/land",
                "0 refs were already backed up",
            ],
            &["refs/heads/zone/land"],
            &[],
        )
    );
}

#[test]
fn an_empty_backup_says_nothing_to_update_and_zero_already_backed_up() {
    let answer = Backup { pushed: Vec::new(), unchanged: Vec::new() };
    assert_eq!(
        backup_report(&answer),
        backed_up(
            &[
                "nothing to update: this host's backup on the remote already holds every in-flight ref",
                "0 refs were already backed up",
            ],
            &[],
            &[],
        )
    );
}
