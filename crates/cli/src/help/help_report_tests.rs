//! help_report: the program help and a verb's help are exit 0, outcome "help", next "mr status", with the "verbs"
//! member; an unknown verb or more than one is help's usage error, exit 4.
//! Unit test: the help groups, the verb table, the usage report and the renderers are pure and used directly.

use super::{find_help_entry, help_report, HelpRequest, HelpTopic, HELP_GROUPS};
use crate::report::{render_json, render_plain, ExitCode, OutValue};

fn verb(name: &str, json: bool) -> HelpRequest {
    let (group, entry) = find_help_entry(name).expect("the verb is in the help");
    HelpRequest { json, topic: HelpTopic::Verb(group, entry) }
}

#[test]
fn the_program_help_is_done_and_next_is_status() {
    let report = help_report(&HelpRequest { json: false, topic: HelpTopic::Program });
    assert_eq!(report.verb, "help");
    assert_eq!(report.exit, ExitCode::Done);
    assert_eq!(report.outcome, "help");
    assert_eq!(report.next, "mr status");
    assert_eq!(report.lines[0], "mr <verb> [args] [--json] [--key <operation key>] [--actor <name>]");
    assert!(render_plain(&report).ends_with("next: mr status\n"));
}

#[test]
fn the_program_help_lists_every_verb_in_its_json() {
    let report = help_report(&HelpRequest { json: true, topic: HelpTopic::Program });
    let [(key, OutValue::Raw(verbs))] = report.fields.as_slice() else {
        panic!("one Raw field: {:?}", report.fields);
    };
    assert_eq!(key, "verbs");
    // 6 agent, 8 folder, 14 landing and 4 setup entries.
    assert_eq!(verbs.len(), 32);
    let actual: Vec<_> = verbs
        .iter()
        .map(|verb| {
            let (name, rest) = verb
                .strip_prefix(r#"{"name":""#)
                .and_then(|entry| entry.split_once(r#"","group":""#))
                .unwrap_or_else(|| panic!("bad verb entry: {verb}"));
            let (group, _) = rest.split_once(r#"",""#).unwrap_or_else(|| panic!("bad verb entry: {verb}"));
            (name, group)
        })
        .collect();
    let expected: Vec<_> = HELP_GROUPS
        .iter()
        .flat_map(|group| group.entries.iter().map(move |entry| (entry.name, group.key)))
        .collect();
    assert_eq!(actual, expected);
}

#[test]
fn a_verb_help_is_done_and_next_is_status() {
    let report = help_report(&verb("status", false));
    assert_eq!(report.exit, ExitCode::Done);
    assert_eq!(report.outcome, "help");
    assert_eq!(report.next, "mr status");
    assert_eq!(
        render_plain(&report),
        "mr status [--json]\nwhere you are, what you changed, whether your zone moved, anything conflicted, and the \
         next step\ntokens saved is counted against: git status and git diff HEAD over this folder\nnext: mr status\n"
    );
}

#[test]
fn a_verb_help_as_json_is_one_object_with_its_entry() {
    assert_eq!(
        render_json(&help_report(&verb("undo", true))),
        concat!(
            r#"{"verb":"help","ok":true,"outcome":"help","exit":0,"next":"mr status","#,
            r#""lines":["mr undo [--key <key>] [--json]","take back your own last step"],"owner":null,"#,
            r#""verbs":[{"name":"undo","group":"agent","line":"take back your own last step","#,
            r#""usage":["mr undo [--key <key>] [--json]"]}]}"#,
            "\n"
        )
    );
}

#[test]
fn an_unknown_verb_is_helps_usage_error() {
    let report = help_report(&HelpRequest { json: false, topic: HelpTopic::Unknown("nosuch".to_string()) });
    assert_eq!(report.verb, "help");
    assert_eq!(report.exit, ExitCode::Usage);
    assert_eq!(report.lines[0], r#"unknown verb "nosuch""#);
    assert!(report.lines[1].starts_with("verbs: "), "{}", report.lines[1]);
    assert_eq!(report.next, "mr help [<verb>] [--json]");
}

#[test]
fn more_than_one_verb_is_helps_usage_error() {
    let report = help_report(&HelpRequest { json: false, topic: HelpTopic::TooMany });
    assert_eq!(report.exit, ExitCode::Usage);
    assert_eq!(report.lines[0], "help takes at most one verb");
    assert_eq!(report.next, "mr help [<verb>] [--json]");
}
