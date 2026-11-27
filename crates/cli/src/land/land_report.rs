//! The report of an `mr land` that land's land_batch answered, with the landing batch read after it.

use std::path::Path;

use ::land::{BatchLandAnswer, BatchMember, Squash};
use gitio::Oid;

use crate::integration_folder::integration_folder_line;
use crate::landing::CHANGE_CONFLICT_NEXT;
use crate::report::{ExitCode, OutValue, Report};

const VERB: &str = "land";

/// `batch` is the landing batch's members in landing order, read after a Landed or Conflicted answer (empty when
/// there is none, and for Busy and ZoneInBatch).
/// - Landed: exit 0 "landed", naming its `integration` as integration's tip and the integration folder (`folder`,
///   None for no folder); the caller gives integration's real tip there, which for a repeat of an earlier member
///   is the batch's tip, not that member's commit. In a batch of more than one, a second line names this squash's
///   place in it. Next runs the integration tests, then publish or back-out. Fields change_id, commit, integration,
///   folder (Null for none) and batch (the batch's change IDs; this squash's alone when the batch is empty).
/// - Conflicted: exit 1 "conflict", each conflicting file on its own line, owner the change's zone; next is
///   CHANGE_CONFLICT_NEXT, and with a batch in flight the last line names it and next says it is kept for another
///   zone's squash. Fields change_id, commit, files and batch; with no batch in flight batch is [], since the
///   conflicting squash did not join one.
/// - Busy: exit 4 "busy", owned by the other change's landing; next finishes that landing, then lands this
///   commit again. Fields change_id, commit and busy_with.
/// - ZoneInBatch: exit 3 "zone-in-batch", owned by the landing of the member holding the zone; next publishes or
///   backs out the batch, then lands this commit again. Fields change_id, commit, zone and member.
pub fn land_report(
    squash: &Squash,
    answer: &BatchLandAnswer,
    batch: &[BatchMember],
    folder: Option<&Path>,
) -> Report {
    let (id, commit) = (&squash.change_id, &squash.commit);
    let mut fields = vec![text("change_id", id.to_string()), text("commit", commit.to_string())];
    let ids: Vec<String> = batch.iter().map(|member| member.change_id.to_string()).collect();
    let (exit, outcome, lines, owner, next) = match answer {
        BatchLandAnswer::Landed { integration } => {
            fields.push(text("integration", integration.to_string()));
            let folder_value = folder.map_or(OutValue::Null, |path| OutValue::Text(path.display().to_string()));
            fields.push(("folder".to_string(), folder_value));
            let landed_batch = if ids.is_empty() { vec![id.to_string()] } else { ids.clone() };
            fields.push(("batch".to_string(), OutValue::Texts(landed_batch)));
            let mut lines = vec![format!("landed change {id} (commit {commit}): integration is at {integration}")];
            lines.extend(batch_place_line(&id.to_string(), &ids));
            lines.push(integration_folder_line(folder, integration));
            let next = "run the integration tests on integration, then mr publish --actor <name> when they pass, \
                        or mr back-out --actor <name> when they fail";
            (ExitCode::Done, "landed", lines, None, next.to_string())
        }
        BatchLandAnswer::Conflicted { files } => {
            let files: Vec<String> = files.iter().map(|file| file.display().to_string()).collect();
            let (lines, next) = conflicted_lines(&id.to_string(), commit, &files, &ids);
            fields.push(("files".to_string(), OutValue::Texts(files)));
            fields.push(("batch".to_string(), OutValue::Texts(ids)));
            (ExitCode::Conflict, "conflict", lines, Some("the change's zone".to_string()), next)
        }
        BatchLandAnswer::Busy { change_id: other } => {
            fields.push(text("busy_with", other.to_string()));
            let line = format!("change {other}'s landing is in flight on integration; change {id} was not landed");
            let next = format!(
                "finish that landing first (mr publish --actor <name>, or mr back-out --actor <name>), \
                 then mr land {commit} --actor <name>"
            );
            (ExitCode::Busy, "busy", vec![line], Some(format!("change {other}'s landing")), next)
        }
        BatchLandAnswer::ZoneInBatch { zone, change_id: member } => {
            fields.push(text("zone", zone.clone()));
            fields.push(text("member", member.to_string()));
            let lines = vec![
                format!(
                    "zone {zone} is already in the batch as change {member}; change {id} (commit {commit}) was not \
                     landed"
                ),
                "integration and the batch were not moved".to_string(),
            ];
            let next = format!(
                "publish or back out the batch first (mr publish --actor <name>, or mr back-out --actor <name>), \
                 then mr land {commit} --actor <name>"
            );
            (ExitCode::Refused, "zone-in-batch", lines, Some(format!("change {member}'s landing")), next)
        }
    };
    Report { verb: VERB.to_string(), exit, outcome: outcome.to_string(), lines, owner, next, fields }
}

/// The line naming this squash's 1-based place in the batch, only in a batch of more than one: a batch of one
/// keeps the single-landing text.
fn batch_place_line(id: &str, ids: &[String]) -> Option<String> {
    if ids.len() <= 1 {
        return None;
    }
    let place = ids.iter().position(|member| member == id)? + 1;
    Some(format!("landed {place} of the batch, in landing order: {}", ids.join(", ")))
}

/// A conflict's lines and next: with no batch in flight, integration and the landing state untouched; with one,
/// the batch named as unchanged and kept for another zone's squash.
fn conflicted_lines(id: &str, commit: &Oid, files: &[String], ids: &[String]) -> (Vec<String>, String) {
    let mut lines = vec![format!("change {id} (commit {commit}) conflicts with integration in:")];
    lines.extend(files.iter().cloned());
    if ids.is_empty() {
        lines.push("integration was not moved and no landing state was written".to_string());
        return (lines, CHANGE_CONFLICT_NEXT.to_string());
    }
    lines.push(format!("integration was not moved and the batch of {} is unchanged: {}", ids.len(), ids.join(", ")));
    let next = format!(
        "{CHANGE_CONFLICT_NEXT}; the batch is kept: land another zone's squash onto it with mr land <commit> \
         --actor <name> (mr would-merge <change id> --actor <name> names a squash's commit)"
    );
    (lines, next)
}

fn text(name: &str, value: String) -> (String, OutValue) {
    (name.to_string(), OutValue::Text(value))
}
