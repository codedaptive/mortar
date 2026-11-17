//! The report of an `mr restore` that oplog::restore answered.

use std::path::PathBuf;

use gitio::Oid;
use oplog::{ApplyOutcome, OpResult, RefMove};

use super::FolderOnRef;
use crate::report::{is_zero_oid, ExitCode, OutValue, Report};

/// Done is exit 0 "restored"; Repeated (the same key ran before) exit 0 "already-done"; both name next
/// "mr log --key <key>". Failed is exit 1 "not-moved": a ref was not where it was read and nothing moved; next
/// "mr log --ref <that ref>". Lines: each move of the record, "<ref> <old|absent> -> <new|deleted>" (or "every
/// ref already holds its value as of op <op_id>" when there is none; for Failed, the ref that stood in the way
/// instead); each folder `moved`; each folder already `in_place`; each folder `left_alone` because its ref has moved
/// on since the record's op (a repeat never reapplies its move after a later mover, S-113(3)), "the folder <path>
/// stays: its ref has moved on since op <record op id>"; each zone folder, whose files stay (S-115(1)).
/// Fields: op_id, key, moves, folders_moved, folders_left and zone_folders.
pub fn restore_report(
    outcome: &ApplyOutcome,
    op_id: &str,
    moved: &[PathBuf],
    in_place: &[PathBuf],
    left_alone: &[PathBuf],
    zone_folders: &[FolderOnRef],
) -> Report {
    let (record, exit, word) = match outcome {
        ApplyOutcome::Done(record) => (record, ExitCode::Done, "restored"),
        ApplyOutcome::Repeated(record) => (record, ExitCode::Done, "already-done"),
        ApplyOutcome::Failed(record) => (record, ExitCode::Refused, "not-moved"),
    };
    let moves: Vec<String> = record.moves.iter().map(move_line).collect();
    let mut lines = match outcome {
        ApplyOutcome::Failed(_) => vec![stale_line(record.result.as_ref(), &record.moves)],
        _ if moves.is_empty() => vec![format!("every ref already holds its value as of op {op_id}")],
        _ => moves.clone(),
    };
    lines.extend(moved.iter().map(|path| format!("moved the folder {} to its ref's restored value", path.display())));
    lines.extend(
        in_place.iter().map(|path| format!("the folder {} already stands at its ref's restored value", path.display())),
    );
    lines.extend(left_alone.iter().map(|path| {
        format!("the folder {} stays: its ref has moved on since op {}", path.display(), record.op_id)
    }));
    lines.extend(zone_folders.iter().map(|folder| {
        let path = folder.path.display();
        format!("the zone folder {path} stands on {}; its files stay where they were", folder.refname)
    }));
    let next = match outcome {
        ApplyOutcome::Failed(_) => format!("mr log --ref {}", stale_ref(record.result.as_ref(), &record.moves)),
        _ => format!("mr log --key {}", record.key),
    };
    Report {
        verb: "restore".to_string(),
        exit,
        outcome: word.to_string(),
        lines,
        owner: None,
        next,
        fields: vec![
            ("op_id".to_string(), OutValue::Text(op_id.to_string())),
            ("key".to_string(), OutValue::Text(record.key.clone())),
            ("moves".to_string(), OutValue::Texts(moves)),
            ("folders_moved".to_string(), OutValue::Texts(moved.iter().map(|p| p.display().to_string()).collect())),
            ("folders_left".to_string(), OutValue::Texts(left_alone.iter().map(|p| p.display().to_string()).collect())),
            (
                "zone_folders".to_string(),
                OutValue::Texts(zone_folders.iter().map(|f| f.path.display().to_string()).collect()),
            ),
        ],
    }
}

fn move_line(m: &RefMove) -> String {
    let old = m.old.as_ref().map_or("absent", Oid::as_str);
    let new = if is_zero_oid(&m.new) { "deleted" } else { m.new.as_str() };
    format!("{} {old} -> {new}", m.refname)
}

/// The first ref the failed record names as not where it was read; the first move's ref when it names none.
fn stale_ref<'a>(result: Option<&'a OpResult>, moves: &'a [RefMove]) -> &'a str {
    let named = match result {
        Some(OpResult::Stale { refname, .. }) => Some(refname),
        Some(OpResult::NotMoved { refs }) => refs.first().map(|(refname, _)| refname),
        _ => None,
    };
    named.or_else(|| moves.first().map(|m| &m.refname)).map_or("", String::as_str)
}

fn stale_line(result: Option<&OpResult>, moves: &[RefMove]) -> String {
    let refname = stale_ref(result, moves);
    let held = match result {
        Some(OpResult::Stale { actual, .. }) => actual.as_ref(),
        Some(OpResult::NotMoved { refs }) => refs.first().and_then(|(_, actual)| actual.as_ref()),
        _ => None,
    };
    let held = held.map_or("nothing", Oid::as_str);
    format!("{refname} was not where it was read (it holds {held}), so nothing moved")
}
