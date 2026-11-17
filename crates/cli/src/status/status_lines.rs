//! status_lines: the plain lines of `mr status`, in the spec's order.

use std::path::PathBuf;

use super::StatusState;
use crate::join::MoveCause;

/// Where the folder stands, one fact per line: the request and its zone; the changed, untracked and conflicted
/// paths (one line per non-empty list, or one line saying there are none); whether everything is saved, then each
/// file the size cap refused in save's wording; whether the zone has commits this request has not taken in, then
/// whether the request is joined; a paused join with its conflicted files; a stalled folder move (S-85), worded for
/// the rebase or the undo that left it (S-143); each overlapping request with the shared paths (S-45); and the gaps.
/// Ids are written by `as_str`.
pub fn status_lines(state: &StatusState) -> Vec<String> {
    let mut lines = vec![format!("request {} in zone {}", state.request, state.zone)];
    let path_lines: Vec<String> = [
        ("changed", &state.paths.changed),
        ("untracked", &state.paths.untracked),
        ("conflicted", &state.paths.conflicted),
    ]
    .iter()
    .filter(|(_, list)| !list.is_empty())
    .map(|(kind, list)| format!("{kind}: {}", joined(list)))
    .collect();
    if path_lines.is_empty() {
        lines.push("no changed, untracked or conflicted paths".to_string());
    }
    lines.extend(path_lines);
    lines.push(if state.saved { "everything is saved" } else { "there is work not yet saved" }.to_string());
    lines.extend(state.refused.iter().map(|file| {
        format!(
            "not saved: {} is {} bytes, over the size cap of {} bytes",
            file.path.display(),
            file.size,
            file.cap
        )
    }));
    lines.push(format!(
        "the zone has {}commits this request has not taken in",
        if state.zone_moved { "" } else { "no " }
    ));
    if state.joined {
        lines.push("the request is joined to the zone".to_string());
    }
    lines.extend(state.paused.iter().map(|paused| {
        format!(
            "a join is paused at commit {} with conflicted files: {}",
            paused.commit.as_str(),
            joined(&paused.files)
        )
    }));
    lines.extend(state.stalled.iter().map(|stalled| {
        let (request_ref, new, old) = (&stalled.request_ref, stalled.new.as_str(), stalled.old.as_str());
        match stalled.cause {
            MoveCause::Rebase => {
                format!("a folder move is stalled: {request_ref} stands at {new}, the folder at {old}")
            }
            MoveCause::Undo => format!(
                "an undo's folder move is stalled: the undo put {request_ref} back at {new}, the folder at {old}"
            ),
        }
    }));
    lines.extend(state.overlaps.iter().map(|(other, paths)| format!("overlaps with {other}: {}", joined(paths))));
    if !state.gaps.is_empty() {
        lines.push(format!("gaps (needed outside the work order): {}", state.gaps.join(", ")));
    }
    lines
}

/// The paths as shown, separated by ", ".
fn joined(paths: &[PathBuf]) -> String {
    paths.iter().map(|path| path.display().to_string()).collect::<Vec<_>>().join(", ")
}
