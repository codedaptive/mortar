//! Who a verb acts as: an agent verb's request, or a harness verb's --actor.

use folders::FolderRecord;

use crate::args::{usage_report, UsageError, VerbKind, VerbSpec};
use crate::report::{ExitCode, Report};

/// Agent verbs act as "<zone>/<request>" from the folder record and refuse --actor (usage, exit 4). A zone
/// folder is refused (exit 1) naming its kind and zone (S-81); no record is refused (exit 1) as not a Mortar
/// folder; both say to run the verb in a request folder. Harness verbs act as --actor, which they require (usage,
/// exit 4). A usage report's next step is `verb`'s own usage line.
pub fn actor_for(verb: &VerbSpec, record: Option<&FolderRecord>, flag: Option<&str>) -> Result<String, Report> {
    let usage = |reason: &str| Report {
        next: verb.usage.to_string(),
        ..usage_report(&UsageError { verb: Some(verb.name.to_string()), reason: reason.to_string(), json: false })
    };
    match (verb.kind, flag) {
        (VerbKind::Agent, Some(_)) => Err(usage("agent verbs take the actor from the folder record")),
        (VerbKind::Agent, None) => match record {
            Some(FolderRecord { zone, request: Some(request), .. }) => Ok(format!("{zone}/{request}")),
            Some(FolderRecord { zone, .. }) => Err(not_a_request_folder(
                verb.name,
                format!("this is a zone folder (zone {zone}): agent verbs run in a request folder"),
            )),
            None => Err(not_a_request_folder(verb.name, "this directory is not in a Mortar folder".to_string())),
        },
        (VerbKind::Harness, Some(actor)) if !actor.is_empty() => Ok(actor.to_string()),
        (VerbKind::Harness, _) => Err(usage("--actor <name> is required")),
    }
}

fn not_a_request_folder(verb: &str, why: String) -> Report {
    Report {
        verb: verb.to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![why, "nothing ran".to_string()],
        owner: Some("the harness that makes Mortar folders".to_string()),
        next: format!("run mortar {verb} inside a request folder Mortar made"),
        fields: Vec::new(),
    }
}
