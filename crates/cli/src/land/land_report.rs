//! The report of a `mortar land` that land's land answered (SPEC-cli 06).

use std::path::Path;

use ::land::{LandAnswer, Squash};

use crate::integration_folder::integration_folder_line;
use crate::landing::CHANGE_CONFLICT_NEXT;
use crate::report::{ExitCode, OutValue, Report};

const VERB: &str = "land";

/// - Landed: exit 0 "landed", naming integration's new tip and whether the integration folder moved (`folder`);
///   next runs the integration tests, then publish or back-out. Fields change_id, commit, integration and folder
///   (Null when none moved).
/// - Conflicted: exit 2 "conflict", each conflicting file on its own line, owner the change's zone; next is
///   CHANGE_CONFLICT_NEXT. Fields change_id, commit and files.
/// - Busy: exit 3 "busy", owned by the other change's landing; next finishes that landing, then lands this
///   commit again. Fields change_id, commit and busy_with.
pub fn land_report(squash: &Squash, answer: &LandAnswer, folder: Option<&Path>) -> Report {
    let (id, commit) = (&squash.change_id, &squash.commit);
    let mut fields = vec![
        ("change_id".to_string(), OutValue::Text(id.to_string())),
        ("commit".to_string(), OutValue::Text(commit.to_string())),
    ];
    match answer {
        LandAnswer::Landed { integration } => {
            fields.push(("integration".to_string(), OutValue::Text(integration.to_string())));
            fields.push((
                "folder".to_string(),
                folder.map_or(OutValue::Null, |path| OutValue::Text(path.display().to_string())),
            ));
            Report {
                verb: VERB.to_string(),
                exit: ExitCode::Done,
                outcome: "landed".to_string(),
                lines: vec![
                    format!("landed change {id} (commit {commit}): integration is at {integration}"),
                    integration_folder_line(folder, integration),
                ],
                owner: None,
                next: "run the integration tests on integration, then mortar publish --actor <name> when they pass, \
                       or mortar back-out --actor <name> when they fail"
                    .to_string(),
                fields,
            }
        }
        LandAnswer::Conflicted { files } => {
            let files: Vec<String> = files.iter().map(|file| file.display().to_string()).collect();
            let mut lines = vec![format!("change {id} (commit {commit}) conflicts with integration in:")];
            lines.extend(files.iter().cloned());
            lines.push("integration was not moved and no landing state was written".to_string());
            fields.push(("files".to_string(), OutValue::Texts(files)));
            Report {
                verb: VERB.to_string(),
                exit: ExitCode::Conflict,
                outcome: "conflict".to_string(),
                lines,
                owner: Some("the change's zone".to_string()),
                next: CHANGE_CONFLICT_NEXT.to_string(),
                fields,
            }
        }
        LandAnswer::Busy { change_id: other } => {
            fields.push(("busy_with".to_string(), OutValue::Text(other.to_string())));
            Report {
                verb: VERB.to_string(),
                exit: ExitCode::Busy,
                outcome: "busy".to_string(),
                lines: vec![format!(
                    "change {other}'s landing is in flight on integration; change {id} was not landed"
                )],
                owner: Some(format!("change {other}'s landing")),
                next: format!(
                    "finish that landing first (mortar publish --actor <name>, or mortar back-out --actor <name>), \
                     then mortar land {commit} --actor <name>"
                ),
                fields,
            }
        }
    }
}
