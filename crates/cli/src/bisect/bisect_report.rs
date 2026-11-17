//! The report of an `mr bisect` that land's bisect answered (SPEC-cli 06).

use gitio::Oid;
use land::{BisectAnswer, BisectNotFound, BisectReport, Culprit};

use super::{culprit_line, probe_line};
use crate::report::{ExitCode, OutValue, Report};

const VERB: &str = "bisect";

/// Every probe, in the order it ran, is a "probe <probe_line>" line after the answer's lines and an entry of the
/// "probes" field.
/// - Culprit: exit 0 "culprit", naming the first failing landing and what its squash trailers carry; next fixes the
///   break through a request in the culprit's zone (a placeholder zone when no Mortar-Zone trailer names it), then
///   squashes that zone. Fields commit, change_id and zone (Null when absent), probes.
/// - NotFound: exit 1 "not-found", owned by the test command: FailedPasses asks to check that the command fails at
///   `failed`; LastPassFails asks for an earlier passing commit. Fields last_pass, failed, probes.
pub fn bisect_report(report: &BisectReport, last_pass: &Oid, failed: &Oid) -> Report {
    let probes: Vec<String> = report.probes.iter().map(probe_line).collect();
    let probe_lines = probes.iter().map(|line| format!("probe {line}"));
    let probes_field = ("probes".to_string(), OutValue::Texts(probes.clone()));
    match &report.answer {
        BisectAnswer::Culprit(culprit) => {
            let mut lines = vec![format!("the first failing landing is {}", culprit.commit), culprit_line(culprit)];
            lines.extend(probe_lines);
            Report {
                verb: VERB.to_string(),
                exit: ExitCode::Done,
                outcome: "culprit".to_string(),
                lines,
                owner: None,
                next: culprit_next(culprit),
                fields: vec![
                    text("commit", culprit.commit.to_string()),
                    text_or_null("change_id", culprit.change_id.as_ref().map(ToString::to_string)),
                    text_or_null("zone", culprit.zone.clone()),
                    probes_field,
                ],
            }
        }
        BisectAnswer::NotFound(why) => {
            let (line, next) = match why {
                BisectNotFound::FailedPasses => (
                    format!("no culprit: the failed commit {failed} passes the test command"),
                    format!("check that the test command fails at {failed}, then repeat the same command"),
                ),
                BisectNotFound::LastPassFails => (
                    format!(
                        "no culprit: the last passing commit {last_pass} fails the test command, so the break is at \
                         or before it"
                    ),
                    format!(
                        "repeat with an earlier passing commit: mr bisect <last pass> {failed} --timeout \
                         <seconds> --actor <name> -- <command>..."
                    ),
                ),
            };
            let mut lines = vec![line];
            lines.extend(probe_lines);
            Report {
                verb: VERB.to_string(),
                exit: ExitCode::Refused,
                outcome: "not-found".to_string(),
                lines,
                owner: Some("the test command".to_string()),
                next,
                fields: vec![
                    text("last_pass", last_pass.to_string()),
                    text("failed", failed.to_string()),
                    probes_field,
                ],
            }
        }
    }
}

/// The fix through a request in the culprit's zone, then that zone's squash.
fn culprit_next(culprit: &Culprit) -> String {
    match &culprit.zone {
        Some(zone) => format!(
            "fix the break in zone {zone} through a request (mr folder request {zone} <request> <path> --paths \
             <path>... --actor <name>), then mr squash {zone} --actor <name>"
        ),
        None => "fix the break in the zone that owns it through a request (mr folder request <zone> <request> \
                 <path> --paths <path>... --actor <name>), then mr squash <zone> --actor <name>"
            .to_string(),
    }
}

fn text(name: &str, value: String) -> (String, OutValue) {
    (name.to_string(), OutValue::Text(value))
}

fn text_or_null(name: &str, value: Option<String>) -> (String, OutValue) {
    (name.to_string(), value.map_or(OutValue::Null, OutValue::Text))
}
