//! squash_pick_report: One is no report; Missing and Several are landing refusals naming the change ID (and the
//! refs carrying it), owned by the zones' squash refs.

use gitio::Oid;
use oplog::ChangeId;

use super::squash_pick_report;
use crate::landing::SquashPick;
use crate::report::{ExitCode, Report};

const CHANGE: &str = "kkkkllllmmmmnnnnooooppppqqqqrrrr";
const COMMIT: &str = "1111111111111111111111111111111111111111";

fn change() -> ChangeId {
    ChangeId::parse(CHANGE).unwrap()
}

#[test]
fn one_is_no_report() {
    let pick = SquashPick::One { refname: "refs/mortar/squash/core".to_string(), commit: Oid::parse(COMMIT).unwrap() };
    assert_eq!(squash_pick_report("would-merge", &change(), &pick), None);
}

#[test]
fn missing_is_a_refusal_that_says_to_make_the_squash() {
    assert_eq!(
        squash_pick_report("would-merge", &change(), &SquashPick::Missing),
        Some(Report {
            verb: "would-merge".to_string(),
            exit: ExitCode::Refused,
            outcome: "refused".to_string(),
            lines: vec![
                "no squash at refs/mortar/squash/* carries change ID kkkkllllmmmmnnnnooooppppqqqqrrrr".to_string()
            ],
            owner: Some("the zones' squash refs".to_string()),
            next: "make the change's squash with mortar squash <zone> --actor <name>, then repeat the same command"
                .to_string(),
            fields: Vec::new(),
        })
    );
}

#[test]
fn several_is_a_refusal_naming_each_ref_and_the_verb_to_repeat_by_commit() {
    let pick = SquashPick::Several {
        refnames: vec!["refs/mortar/squash/web".to_string(), "refs/mortar/squash/core".to_string()],
    };
    assert_eq!(
        squash_pick_report("land", &change(), &pick),
        Some(Report {
            verb: "land".to_string(),
            exit: ExitCode::Refused,
            outcome: "refused".to_string(),
            lines: vec![
                "change ID kkkkllllmmmmnnnnooooppppqqqqrrrr is carried by more than one squash ref: \
                 refs/mortar/squash/web, refs/mortar/squash/core"
                    .to_string()
            ],
            owner: Some("the zones' squash refs".to_string()),
            next: "name the squash by its commit instead: mortar land <commit> --actor <name>".to_string(),
            fields: Vec::new(),
        })
    );
}

#[test]
fn several_names_the_verb_it_was_given() {
    let pick = SquashPick::Several {
        refnames: vec!["refs/mortar/squash/a".to_string(), "refs/mortar/squash/b".to_string()],
    };
    let report = squash_pick_report("would-merge", &change(), &pick).expect("a refusal");
    assert_eq!(report.verb, "would-merge");
    assert_eq!(report.next, "name the squash by its commit instead: mortar would-merge <commit> --actor <name>");
}
