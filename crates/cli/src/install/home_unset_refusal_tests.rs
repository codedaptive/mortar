//! Refusal when HOME is not set.

use crate::report::{ExitCode, Report};
use super::home_unset_refusal;

#[test]
fn refusal_names_verb_and_owner() {
    assert_eq!(home_unset_refusal("uninstall", "the person uninstalling Mortar"), Report {
        verb: "uninstall".into(), exit: ExitCode::Refused, outcome: "refused".into(),
        lines: vec!["HOME is not set or is empty".into(), "nothing changed".into()],
        owner: Some("the person uninstalling Mortar".into()), next: "set HOME, then repeat".into(), fields: vec![],
    });
}
