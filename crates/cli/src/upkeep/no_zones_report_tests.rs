//! no_zones_report: a config with no zone is refused before any git (exit 1), naming the danger of an empty zone list
//! and the init step that mends the config.

use super::no_zones_report;
use crate::report::{ExitCode, Report};

fn expected(verb: &str) -> Report {
    Report {
        verb: verb.to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![
            "the config names no zone".to_string(),
            "with no zone listed, a sweep would delete every zone's replay records (refs/mortar/replayed/); nothing \
             was read or deleted"
                .to_string(),
        ],
        owner: Some("the repository's mortar config".to_string()),
        next: "mortar init --replace with every value the config holds and at least one --zone <zone>=<path> --actor \
               <name>, then repeat the same command"
            .to_string(),
        fields: Vec::new(),
    }
}

#[test]
fn sweep_with_no_zones_is_refused() {
    assert_eq!(no_zones_report("sweep"), expected("sweep"));
}

#[test]
fn disk_with_no_zones_is_refused_under_its_own_verb() {
    assert_eq!(no_zones_report("disk"), expected("disk"));
}

#[test]
fn the_refusal_is_exit_one() {
    assert_eq!(no_zones_report("sweep").exit as i32, 1);
}
