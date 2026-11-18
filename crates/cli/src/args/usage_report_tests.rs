//! usage_report: exit 4, the reason then the verbs, and the verb's usage (or the verb list) as next.
//! ZUT (rule 11): args/find_verb and the verb table are pure and used directly.

use super::{usage_report, UsageError};
use crate::report::{ExitCode, Report};

const INIT_USAGE: &str = "mr init --actor <name> --origin <branch> --integration <branch> --remote <name> \
[--zone <zone>=<path>[,<path>...]]... [--root-file <path>]... [--build-dir <path>]... [--cap-bytes <n>] \
[--floor-bytes <n>] [--formatter <word>]... [--formatter-timeout <seconds>] [--replace] [--json]";

#[test]
fn an_unknown_verb_lists_the_verbs() {
    let error = UsageError { verb: Some("jump".to_string()), reason: "unknown verb \"jump\"".to_string(), json: true };
    assert_eq!(
        usage_report(&error),
        Report {
            verb: "jump".to_string(),
            exit: ExitCode::Usage,
            outcome: "usage".to_string(),
            lines: vec!["unknown verb \"jump\"".to_string(), "verbs: after-publish, back-out, backup, bisect, disk, folder, guard, guard-hook, init, join, land, log, need, overlap, publish, rereview, resolve, restore, restore-backup, save, snapshot, squash, status, sweep, undo, would-merge".to_string()],
            owner: None,
            next: "mr <verb>, one of: after-publish, back-out, backup, bisect, disk, folder, guard, guard-hook, init, join, land, log, need, overlap, publish, rereview, resolve, restore, restore-backup, save, snapshot, squash, status, sweep, undo, would-merge".to_string(),
            fields: Vec::new(),
        }
    );
}

#[test]
fn no_verb_has_an_empty_verb() {
    let report = usage_report(&UsageError { verb: None, reason: "no verb given".to_string(), json: false });
    assert_eq!(report.verb, "");
    assert_eq!(report.lines, vec!["no verb given".to_string(), "verbs: after-publish, back-out, backup, bisect, disk, folder, guard, guard-hook, init, join, land, log, need, overlap, publish, rereview, resolve, restore, restore-backup, save, snapshot, squash, status, sweep, undo, would-merge".to_string()]);
    assert_eq!(report.next, "mr <verb>, one of: after-publish, back-out, backup, bisect, disk, folder, guard, guard-hook, init, join, land, log, need, overlap, publish, rereview, resolve, restore, restore-backup, save, snapshot, squash, status, sweep, undo, would-merge");
}

#[test]
fn a_known_verb_gives_its_usage_as_next() {
    let error = UsageError { verb: Some("init".to_string()), reason: "--key needs a value".to_string(), json: false };
    let report = usage_report(&error);
    assert_eq!(report.verb, "init");
    assert_eq!(report.exit as i32, 4);
    assert_eq!(report.next, INIT_USAGE);
}
