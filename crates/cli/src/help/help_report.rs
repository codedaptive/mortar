//! help_report: the report that answers a help request.

use super::{help_entry_json, program_help_lines, verb_help_lines, HelpRequest, HelpTopic, HELP_GROUPS};
use crate::args::{usage_report, UsageError};
use crate::report::{ExitCode, OutValue, Report};

/// The program help or one verb's: exit Done, outcome "help", next "mr status", and the `--json` member "verbs"
/// holding every verb's entry in group order, or the one verb's. An unknown verb or more than one is a usage
/// error of `mr help` (exit 4, the verbs listed, next help's usage).
pub fn help_report(request: &HelpRequest) -> Report {
    let reason = match &request.topic {
        HelpTopic::Program => {
            let verbs = HELP_GROUPS
                .iter()
                .flat_map(|group| group.entries.iter().map(move |entry| help_entry_json(group, entry)))
                .collect();
            return answer(program_help_lines(), verbs);
        }
        HelpTopic::Verb(group, entry) => return answer(verb_help_lines(entry), vec![help_entry_json(group, entry)]),
        HelpTopic::Unknown(word) => format!("unknown verb {word:?}"),
        HelpTopic::TooMany => "help takes at most one verb".to_string(),
    };
    usage_report(&UsageError { verb: Some("help".to_string()), reason, json: request.json })
}

fn answer(lines: Vec<String>, verbs: Vec<String>) -> Report {
    Report {
        verb: "help".to_string(),
        exit: ExitCode::Done,
        outcome: "help".to_string(),
        lines,
        owner: None,
        next: "mr status".to_string(),
        fields: vec![("verbs".to_string(), OutValue::Raw(verbs))],
    }
}
