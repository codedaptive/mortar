//! after_publish_report: the report of an after-publish that land answered, each named zone's move and folder in
//! order.

use std::path::PathBuf;

use gitio::Oid;
use land::ZoneMove;

use super::{repeat_command, zone_move_lines, AfterPublishArgs};
use crate::report::{ExitCode, OutValue, Report};
use crate::zone_folder_move::{folder_part, zone_folder_lines, ZoneFolderOutcome, FOLDER_PARTS};

const VERB: &str = "after-publish";

/// Lines: per zone in order, zone_move_lines then zone_folder_lines at the zone's tip after land's answer. Exit
/// Failure ("folder-failed") when any folder failed, else Refused ("folder-not-moved") when any was refused, else
/// Done ("after-published"); the owner is the folders listed unless Done. next: the folder parts that apply and,
/// when any zone conflicted, its coordinator's part, joined "; " and followed by the repeat; with no part, a join
/// for the requests to rebase, or nothing to do. Fields: the zones moved, current and conflicted, the requests to
/// rebase, and the folders moved, refused and failed.
pub fn after_publish_report(args: &AfterPublishArgs, zones: &[(ZoneMove, ZoneFolderOutcome)]) -> Report {
    let folders = |kind: usize| -> Vec<String> {
        zones
            .iter()
            .filter_map(|(_, outcome)| folder_field(outcome))
            .filter(|(listed, _)| *listed == kind)
            .map(|(_, folder)| folder.display().to_string())
            .collect()
    };
    let (moved, refused, failed) = (folders(0), folders(1), folders(2));
    let (exit, outcome) = match (failed.is_empty(), refused.is_empty()) {
        (false, _) => (ExitCode::Failure, "folder-failed"),
        (true, false) => (ExitCode::Refused, "folder-not-moved"),
        (true, true) => (ExitCode::Done, "after-published"),
    };
    let named = |kind: usize| -> Vec<String> {
        zones
            .iter()
            .map(|(zone_move, _)| zone_field(zone_move))
            .filter(|(listed, ..)| *listed == kind)
            .map(|(_, zone, _)| zone.clone())
            .collect()
    };
    let rebase: Vec<String> = zones.iter().flat_map(|(zone_move, _)| rebase_names(zone_move)).collect();
    let texts = |name: &str, values: Vec<String>| (name.to_string(), OutValue::Texts(values));
    Report {
        verb: VERB.to_string(),
        exit,
        outcome: outcome.to_string(),
        lines: zones
            .iter()
            .flat_map(|(zone_move, folder)| {
                let (_, zone, tip) = zone_field(zone_move);
                zone_move_lines(zone_move).into_iter().chain(zone_folder_lines(zone, tip, folder))
            })
            .collect(),
        owner: (exit != ExitCode::Done).then(|| "the zone folders listed".to_string()),
        next: next(args, zones, &named(2), rebase.is_empty()),
        fields: vec![
            texts("moved", named(0)),
            texts("current", named(1)),
            texts("conflicted", named(2)),
            texts("rebase", rebase),
            texts("folders_moved", moved),
            texts("folders_refused", refused),
            texts("folders_failed", failed),
        ],
    }
}

/// The folder parts that apply, then the conflicted zones' part, joined "; " and followed by the repeat; with no
/// part, a join when any request rebases, else nothing to do.
fn next(
    args: &AfterPublishArgs,
    zones: &[(ZoneMove, ZoneFolderOutcome)],
    conflicted: &[String],
    no_rebase: bool,
) -> String {
    let applies: Vec<usize> = zones.iter().filter_map(|(_, outcome)| folder_part(outcome)).collect();
    let mut parts: Vec<String> = FOLDER_PARTS
        .iter()
        .enumerate()
        .filter(|(part, _)| applies.contains(part))
        .map(|(_, text)| text.to_string())
        .collect();
    if !conflicted.is_empty() {
        let zones = conflicted.join(", ");
        parts.push(format!("zone {zones} conflicts with origin: its coordinator resolves the conflict"));
    }
    match (parts.is_empty(), no_rebase) {
        (false, _) => format!("{}; then repeat the same command: {}", parts.join("; "), repeat_command(args)),
        (true, false) => "run mr join in the folder of each request listed".to_string(),
        (true, true) => "nothing to do".to_string(),
    }
}

/// The field a zone is listed under (0 moved, 1 current, 2 conflicted), its name, and its branch's tip after land's
/// answer.
fn zone_field(zone_move: &ZoneMove) -> (usize, &String, &Oid) {
    match zone_move {
        ZoneMove::Moved { zone, new, .. } => (0, zone, new),
        ZoneMove::Current { zone, tip } => (1, zone, tip),
        ZoneMove::Conflicted { zone, tip, .. } => (2, zone, tip),
    }
}

/// The open requests a moved zone rebases; none for a zone whose branch did not move.
fn rebase_names(zone_move: &ZoneMove) -> Vec<String> {
    match zone_move {
        ZoneMove::Moved { rebase, .. } => rebase.iter().map(|request| request.name.clone()).collect(),
        ZoneMove::Current { .. } | ZoneMove::Conflicted { .. } => Vec::new(),
    }
}

/// The folder field a folder is listed under (0 moved, 1 refused, 2 failed) with its path; None for no folder or
/// one already at the tip.
fn folder_field(outcome: &ZoneFolderOutcome) -> Option<(usize, &PathBuf)> {
    match outcome {
        ZoneFolderOutcome::NoFolder | ZoneFolderOutcome::AtTip { .. } => None,
        ZoneFolderOutcome::Moved { folder } => Some((0, folder)),
        ZoneFolderOutcome::Refused { folder, .. } => Some((1, folder)),
        ZoneFolderOutcome::Failed { folder, .. } => Some((2, folder)),
    }
}
