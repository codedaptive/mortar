//! after_publish_report: each zone's move and folder lines in order; exit, outcome and owner from the folders
//! (Failed beats Refused beats Done); next from the parts that apply in their fixed order, then a repeat; the fields.

use std::path::PathBuf;

use gitio::Oid;
use join::OpenRequest;
use land::ZoneMove;

use super::after_publish_report;
use crate::after_publish::AfterPublishArgs;
use crate::report::{ExitCode, OutValue, Report};
use crate::zone_folder_move::{FolderRefusal, ZoneFolderOutcome};

const OLD: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";
const NEW: &str = "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391";
const TIP: &str = "9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e";
const COMMIT: &str = "3a7d1c9e5b2f8a4d6c0e9b1f7a3d5c8e2b4f6a90";

const IN_THE_WAY: &str = "move each path listed as in the way out of its zone folder, or put it back to that folder's \
                          index copy, and leave the index as it is";
const UNMERGED: &str =
    "finish or undo the merge in each zone folder whose index has unmerged entries, leaving no unmerged entry";
const NO_ZONE_COMMIT: &str = "take the staged changes out of each zone folder whose index matches no zone commit, \
                              without resetting its index to HEAD alone";
const FAILED: &str = "clear what stopped git in each zone folder that failed";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn args(published: &str, moves: &[&str]) -> AfterPublishArgs {
    AfterPublishArgs { published: published.to_string(), moves: moves.iter().map(ToString::to_string).collect() }
}

fn moved(zone: &str, rebase: &[&str]) -> ZoneMove {
    ZoneMove::Moved {
        zone: zone.to_string(),
        old: oid(OLD),
        new: oid(NEW),
        rebase: rebase
            .iter()
            .map(|name| OpenRequest { name: name.to_string(), base: oid(OLD), tip: oid(COMMIT) })
            .collect(),
    }
}

fn current(zone: &str) -> ZoneMove {
    ZoneMove::Current { zone: zone.to_string(), tip: oid(TIP) }
}

fn conflicted(zone: &str) -> ZoneMove {
    ZoneMove::Conflicted {
        zone: zone.to_string(),
        tip: oid(TIP),
        commit: oid(COMMIT),
        files: vec![PathBuf::from("Cargo.lock")],
    }
}

fn folder(zone: &str) -> PathBuf {
    PathBuf::from(format!("/work/zones/{zone}"))
}

fn refused(zone: &str, why: FolderRefusal) -> ZoneFolderOutcome {
    ZoneFolderOutcome::Refused { folder: folder(zone), why }
}

fn failed(zone: &str) -> ZoneFolderOutcome {
    ZoneFolderOutcome::Failed { folder: folder(zone), why: "git could not read its index".to_string() }
}

fn in_the_way() -> FolderRefusal {
    FolderRefusal::InTheWay(vec![PathBuf::from("src/a.rs")])
}

fn texts(name: &str, values: &[&str]) -> (String, OutValue) {
    (name.to_string(), OutValue::Texts(values.iter().map(ToString::to_string).collect()))
}

fn strings(lines: &[&str]) -> Vec<String> {
    lines.iter().map(ToString::to_string).collect()
}

#[test]
fn every_zone_done_lists_each_zone_in_order_and_has_nothing_to_do() {
    let zones = [
        (moved("cli", &[]), ZoneFolderOutcome::Moved { folder: folder("cli") }),
        (current("land"), ZoneFolderOutcome::AtTip { folder: folder("land") }),
        (moved("join", &[]), ZoneFolderOutcome::NoFolder),
    ];
    assert_eq!(
        after_publish_report(&args("cli", &["land", "join"]), &zones),
        Report {
            verb: "after-publish".to_string(),
            exit: ExitCode::Done,
            outcome: "after-published".to_string(),
            lines: strings(&[
                "zone cli moved from 4b825dc642cb6eb9a060e54bf8d69288fbee4904 to \
                 e69de29bb2d1d6434b8b29ae775ad8c2e48c5391",
                "brought the zone folder /work/zones/cli to e69de29bb2d1d6434b8b29ae775ad8c2e48c5391",
                "zone land already holds origin at 9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e; its branch did not move",
                "the zone folder /work/zones/land is already at 9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e",
                "zone join moved from 4b825dc642cb6eb9a060e54bf8d69288fbee4904 to \
                 e69de29bb2d1d6434b8b29ae775ad8c2e48c5391",
                "zone join has no zone folder",
            ]),
            owner: None,
            next: "nothing to do".to_string(),
            fields: vec![
                texts("moved", &["cli", "join"]),
                texts("current", &["land"]),
                texts("conflicted", &[]),
                texts("rebase", &[]),
                texts("folders_moved", &["/work/zones/cli"]),
                texts("folders_refused", &[]),
                texts("folders_failed", &[]),
            ],
        }
    );
}

#[test]
fn requests_to_rebase_send_them_to_join() {
    let zones = [
        (moved("cli", &["r1", "r2"]), ZoneFolderOutcome::AtTip { folder: folder("cli") }),
        (moved("land", &["r3"]), ZoneFolderOutcome::NoFolder),
    ];
    let report = after_publish_report(&args("cli", &["land"]), &zones);
    assert_eq!(report.exit, ExitCode::Done);
    assert_eq!(report.outcome, "after-published");
    assert_eq!(report.owner, None);
    assert_eq!(report.next, "run mr join in the folder of each request listed");
    assert_eq!(report.fields[3], texts("rebase", &["r1", "r2", "r3"]));
    assert_eq!(
        report.lines[1..3],
        strings(&["request r1 rebases at its next join", "request r2 rebases at its next join"])
    );
}

#[test]
fn a_conflicted_zone_alone_is_done_with_its_coordinator_and_the_repeat_next() {
    let zones = [
        (moved("cli", &["r1"]), ZoneFolderOutcome::Moved { folder: folder("cli") }),
        (conflicted("land"), ZoneFolderOutcome::Moved { folder: folder("land") }),
    ];
    assert_eq!(
        after_publish_report(&args("cli", &["land"]), &zones),
        Report {
            verb: "after-publish".to_string(),
            exit: ExitCode::Done,
            outcome: "after-published".to_string(),
            lines: strings(&[
                "zone cli moved from 4b825dc642cb6eb9a060e54bf8d69288fbee4904 to \
                 e69de29bb2d1d6434b8b29ae775ad8c2e48c5391",
                "request r1 rebases at its next join",
                "brought the zone folder /work/zones/cli to e69de29bb2d1d6434b8b29ae775ad8c2e48c5391",
                "zone land left in place at 9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e: replaying \
                 3a7d1c9e5b2f8a4d6c0e9b1f7a3d5c8e2b4f6a90 onto origin conflicts in:",
                "Cargo.lock",
                "brought the zone folder /work/zones/land to 9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e",
            ]),
            owner: None,
            next: "zone land conflicts with origin: its coordinator resolves the conflict; then repeat the same \
                   command: mr after-publish cli --move land --actor <name>"
                .to_string(),
            fields: vec![
                texts("moved", &["cli"]),
                texts("current", &[]),
                texts("conflicted", &["land"]),
                texts("rebase", &["r1"]),
                texts("folders_moved", &["/work/zones/cli", "/work/zones/land"]),
                texts("folders_refused", &[]),
                texts("folders_failed", &[]),
            ],
        }
    );
}

#[test]
fn several_conflicted_zones_are_named_together() {
    let zones = [
        (conflicted("cli"), ZoneFolderOutcome::NoFolder),
        (current("join"), ZoneFolderOutcome::NoFolder),
        (conflicted("land"), ZoneFolderOutcome::NoFolder),
    ];
    let report = after_publish_report(&args("cli", &["join", "land"]), &zones);
    assert_eq!(report.exit, ExitCode::Done);
    assert_eq!(
        report.next,
        "zone cli, land conflicts with origin: its coordinator resolves the conflict; then repeat the same command: \
         mr after-publish cli --move join --move land --actor <name>"
    );
}

#[test]
fn a_folder_with_paths_in_the_way_is_refused_with_its_next_and_the_repeat() {
    let zones = [
        (current("cli"), refused("cli", in_the_way())),
        (moved("land", &[]), ZoneFolderOutcome::Moved { folder: folder("land") }),
    ];
    assert_eq!(
        after_publish_report(&args("cli", &["land"]), &zones),
        Report {
            verb: "after-publish".to_string(),
            exit: ExitCode::Refused,
            outcome: "folder-not-moved".to_string(),
            lines: strings(&[
                "zone cli already holds origin at 9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e; its branch did not move",
                "the zone folder /work/zones/cli was not brought to 9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e: local \
                 changes are in the way at:",
                "src/a.rs",
                "nothing in it changed",
                "zone land moved from 4b825dc642cb6eb9a060e54bf8d69288fbee4904 to \
                 e69de29bb2d1d6434b8b29ae775ad8c2e48c5391",
                "brought the zone folder /work/zones/land to e69de29bb2d1d6434b8b29ae775ad8c2e48c5391",
            ]),
            owner: Some("the zone folders listed".to_string()),
            next: format!(
                "{IN_THE_WAY}; then repeat the same command: mr after-publish cli --move land --actor <name>"
            ),
            fields: vec![
                texts("moved", &["land"]),
                texts("current", &["cli"]),
                texts("conflicted", &[]),
                texts("rebase", &[]),
                texts("folders_moved", &["/work/zones/land"]),
                texts("folders_refused", &["/work/zones/cli"]),
                texts("folders_failed", &[]),
            ],
        }
    );
}

#[test]
fn an_unmerged_folder_is_refused_with_its_next() {
    let unmerged = FolderRefusal::Unmerged(vec![PathBuf::from("Cargo.lock")]);
    let zones = [(moved("cli", &["r1"]), refused("cli", unmerged))];
    let report = after_publish_report(&args("cli", &[]), &zones);
    assert_eq!(report.exit, ExitCode::Refused);
    assert_eq!(report.outcome, "folder-not-moved");
    assert_eq!(report.owner, Some("the zone folders listed".to_string()));
    assert_eq!(report.next, format!("{UNMERGED}; then repeat the same command: mr after-publish cli --actor <name>"));
}

#[test]
fn a_folder_matching_no_zone_commit_is_refused_with_its_next() {
    let zones = [(current("cli"), refused("cli", FolderRefusal::NoZoneCommit))];
    let report = after_publish_report(&args("cli", &[]), &zones);
    assert_eq!(report.exit, ExitCode::Refused);
    assert_eq!(report.outcome, "folder-not-moved");
    assert_eq!(report.owner, Some("the zone folders listed".to_string()));
    assert_eq!(
        report.next,
        format!("{NO_ZONE_COMMIT}; then repeat the same command: mr after-publish cli --actor <name>")
    );
    assert_eq!(report.fields[5], texts("folders_refused", &["/work/zones/cli"]));
}

#[test]
fn a_failed_folder_is_a_failure_with_its_next() {
    let zones = [
        (moved("cli", &[]), failed("cli")),
        (current("land"), ZoneFolderOutcome::AtTip { folder: folder("land") }),
    ];
    let report = after_publish_report(&args("cli", &["land"]), &zones);
    assert_eq!(report.exit, ExitCode::Failure);
    assert_eq!(report.outcome, "folder-failed");
    assert_eq!(report.owner, Some("the zone folders listed".to_string()));
    assert_eq!(
        report.next,
        format!("{FAILED}; then repeat the same command: mr after-publish cli --move land --actor <name>")
    );
    assert_eq!(
        report.lines[1],
        "the zone folder /work/zones/cli was not brought to e69de29bb2d1d6434b8b29ae775ad8c2e48c5391: git could not \
         read its index"
    );
    assert_eq!(report.fields[4], texts("folders_moved", &[]));
    assert_eq!(report.fields[5], texts("folders_refused", &[]));
    assert_eq!(report.fields[6], texts("folders_failed", &["/work/zones/cli"]));
}

#[test]
fn a_failed_folder_beats_a_refused_one_in_either_order() {
    let refused_first = [(current("cli"), refused("cli", in_the_way())), (current("land"), failed("land"))];
    let failed_first = [(current("cli"), failed("cli")), (current("land"), refused("land", in_the_way()))];
    for zones in [refused_first, failed_first] {
        let report = after_publish_report(&args("cli", &["land"]), &zones);
        assert_eq!(report.exit, ExitCode::Failure);
        assert_eq!(report.outcome, "folder-failed");
        assert_eq!(report.owner, Some("the zone folders listed".to_string()));
    }
}

#[test]
fn every_part_that_applies_is_in_the_fixed_order_whatever_the_zones_order() {
    let zones = [
        (conflicted("cli"), ZoneFolderOutcome::AtTip { folder: folder("cli") }),
        (moved("land", &["r1"]), failed("land")),
        (current("join"), refused("join", FolderRefusal::NoZoneCommit)),
        (moved("oplog", &[]), refused("oplog", FolderRefusal::Unmerged(vec![PathBuf::from("x.rs")]))),
        (current("gitio"), refused("gitio", in_the_way())),
    ];
    let report = after_publish_report(&args("cli", &["land", "join", "oplog", "gitio"]), &zones);
    assert_eq!(
        report.next,
        format!(
            "{IN_THE_WAY}; {UNMERGED}; {NO_ZONE_COMMIT}; {FAILED}; zone cli conflicts with origin: its coordinator \
             resolves the conflict; then repeat the same command: mr after-publish cli --move land --move join \
             --move oplog --move gitio --actor <name>"
        )
    );
    assert_eq!(report.exit, ExitCode::Failure);
    assert_eq!(
        report.fields[5],
        texts("folders_refused", &["/work/zones/join", "/work/zones/oplog", "/work/zones/gitio"])
    );
    assert_eq!(report.fields[6], texts("folders_failed", &["/work/zones/land"]));
}

#[test]
fn a_kind_shared_by_several_folders_is_one_part() {
    let zones = [(current("cli"), refused("cli", in_the_way())), (current("land"), refused("land", in_the_way()))];
    let report = after_publish_report(&args("cli", &["land"]), &zones);
    assert_eq!(
        report.next,
        format!("{IN_THE_WAY}; then repeat the same command: mr after-publish cli --move land --actor <name>")
    );
    assert_eq!(report.fields[5], texts("folders_refused", &["/work/zones/cli", "/work/zones/land"]));
}
