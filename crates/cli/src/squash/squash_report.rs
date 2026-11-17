//! The report of a `mortar squash` that land's squash answered (SPEC-cli 06).

use land::{Squash, SquashAnswer};

use crate::report::{ExitCode, OutValue, Report};

const VERB: &str = "squash";

/// - Made: exit 0 "squashed", the squash commit, its change ID, the zone tip it squashes and its requests.
/// - Same: exit 0 "already-squashed", the commit already at the squash ref; nothing written. Fields and next as Made.
/// - Nothing: exit 0 "nothing-to-squash", the zone holds nothing origin lacks; fields zone and source.
/// - Conflicted: exit 2 "conflict", each conflicting file on its own line, owner the zone; next brings the zone up
///   to date through a request, then squashes again. Fields zone and files.
pub fn squash_report(zone: &str, answer: &SquashAnswer) -> Report {
    let report = |exit, outcome: &str, lines, owner, next: String, fields| Report {
        verb: VERB.to_string(),
        exit,
        outcome: outcome.to_string(),
        lines,
        owner,
        next,
        fields,
    };
    let zone_field = ("zone".to_string(), OutValue::Text(zone.to_string()));
    match answer {
        SquashAnswer::Made(squash) => {
            let first = format!(
                "squashed zone {zone} as change {}: commit {}",
                squash.change_id, squash.commit
            );
            let lines = vec![first, format!("it squashes zone tip {}", squash.source), requests_line(squash)];
            report(ExitCode::Done, "squashed", lines, None, review_next(squash), squash_fields(zone_field, squash))
        }
        SquashAnswer::Same(squash) => {
            let first = format!(
                "zone {zone}'s squash is already commit {} (change {}); nothing was written",
                squash.commit, squash.change_id
            );
            let lines = vec![first, requests_line(squash)];
            let fields = squash_fields(zone_field, squash);
            report(ExitCode::Done, "already-squashed", lines, None, review_next(squash), fields)
        }
        SquashAnswer::Nothing { source } => report(
            ExitCode::Done,
            "nothing-to-squash",
            vec![format!("zone {zone} at {source} holds nothing origin lacks; nothing was written")],
            None,
            "nothing to do".to_string(),
            vec![zone_field, ("source".to_string(), OutValue::Text(source.to_string()))],
        ),
        SquashAnswer::Conflicted { files } => {
            let files: Vec<String> = files.iter().map(|file| file.display().to_string()).collect();
            let mut lines = vec![format!("zone {zone} conflicts with origin in:")];
            lines.extend(files.iter().cloned());
            lines.push("nothing was written and no ref moved".to_string());
            let next = format!(
                "bring zone {zone} up to date with origin through a request (mortar folder request {zone} <request> \
                 <path> --paths <path>... --actor <name>), then mortar squash {zone} --actor <name>"
            );
            let fields = vec![zone_field, ("files".to_string(), OutValue::Texts(files))];
            report(ExitCode::Conflict, "conflict", lines, Some(format!("zone {zone}")), next, fields)
        }
    }
}

/// "requests: " and the squash's requests joined ", ", or "none named" when it names none.
fn requests_line(squash: &Squash) -> String {
    if squash.requests.is_empty() {
        return "requests: none named".to_string();
    }
    format!("requests: {}", squash.requests.join(", "))
}

fn review_next(squash: &Squash) -> String {
    format!("hand change {} (commit {}) to review", squash.change_id, squash.commit)
}

/// zone, change_id, commit and source as Text, then requests as Texts.
fn squash_fields(zone_field: (String, OutValue), squash: &Squash) -> Vec<(String, OutValue)> {
    vec![
        zone_field,
        ("change_id".to_string(), OutValue::Text(squash.change_id.to_string())),
        ("commit".to_string(), OutValue::Text(squash.commit.to_string())),
        ("source".to_string(), OutValue::Text(squash.source.to_string())),
        ("requests".to_string(), OutValue::Texts(squash.requests.clone())),
    ]
}
