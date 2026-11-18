//! step_record: The record item a step leaves once it is applied.

use super::{content_hash, InstallStep, RecordItem};

/// PlaceBinary: its file with `binary_hash`. Link: the link and its target. WriteSkill: the file with its text's hash.
pub fn step_record(step: &InstallStep, binary_hash: &str) -> RecordItem {
    match step {
        InstallStep::PlaceBinary { to, .. } => RecordItem::File { path: to.clone(), hash: binary_hash.to_string() },
        InstallStep::Link { at, target } => RecordItem::Link { path: at.clone(), target: target.clone() },
        InstallStep::WriteSkill { path, text } => {
            RecordItem::File { path: path.clone(), hash: content_hash(text.as_bytes()) }
        }
    }
}
