//! status_fields: the --json members of `mortar status`, in the spec's order.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::StatusState;
use crate::report::OutValue;

/// zone and request (Text); changed, untracked and conflicted (Texts); saved (Bool); refused (the paths) and cap
/// (the first refused file's cap, Null when none, as save reports it); zone_moved and joined (Bool); paused_commit
/// (Null when no join is paused) and paused_files; stalled_from and stalled_to, the folder's and the branch's
/// commits of a stalled move (Null when none); overlaps (the other requests, in order) and overlap_paths (every
/// shared path once, sorted); gaps.
pub fn status_fields(state: &StatusState) -> Vec<(String, OutValue)> {
    let fields = [
        ("zone", OutValue::Text(state.zone.clone())),
        ("request", OutValue::Text(state.request.clone())),
        ("changed", texts(&state.paths.changed)),
        ("untracked", texts(&state.paths.untracked)),
        ("conflicted", texts(&state.paths.conflicted)),
        ("saved", OutValue::Bool(state.saved)),
        ("refused", OutValue::Texts(state.refused.iter().map(|file| shown(&file.path)).collect())),
        ("cap", state.refused.first().map_or(OutValue::Null, |file| OutValue::Number(file.cap))),
        ("zone_moved", OutValue::Bool(state.zone_moved)),
        ("joined", OutValue::Bool(state.joined)),
        ("paused_commit", oid_text(state.paused.as_ref().map(|paused| paused.commit.as_str()))),
        ("paused_files", state.paused.as_ref().map_or(OutValue::Texts(Vec::new()), |paused| texts(&paused.files))),
        ("stalled_from", oid_text(state.stalled.as_ref().map(|stalled| stalled.old.as_str()))),
        ("stalled_to", oid_text(state.stalled.as_ref().map(|stalled| stalled.new.as_str()))),
        ("overlaps", OutValue::Texts(state.overlaps.iter().map(|(other, _)| other.clone()).collect())),
        (
            "overlap_paths",
            OutValue::Texts(
                state
                    .overlaps
                    .iter()
                    .flat_map(|(_, paths)| paths.iter().map(|path| shown(path)))
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect(),
            ),
        ),
        ("gaps", OutValue::Texts(state.gaps.clone())),
    ];
    fields.into_iter().map(|(name, value)| (name.to_string(), value)).collect()
}

fn shown(path: &Path) -> String {
    path.display().to_string()
}

fn texts(paths: &[PathBuf]) -> OutValue {
    OutValue::Texts(paths.iter().map(|path| shown(path)).collect())
}

/// An id's text, or Null when there is none.
fn oid_text(hex: Option<&str>) -> OutValue {
    hex.map_or(OutValue::Null, |hex| OutValue::Text(hex.to_string()))
}
