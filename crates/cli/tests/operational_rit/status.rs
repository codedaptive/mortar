use crate::{json::check, observe::*, support::*};
use std::fs;

#[test]
fn rit_status_reads_clean_saved_unjoined_and_joined_tips_without_writing() {
    // mortar status -> read_tips/saved_state/ancestry -> literal next/fields, both branch states.
    let f = Fixture::new(); f.save_topic();
    let saved = status(&f, "saved");
    assert!(saved.flag("saved")); assert!(!saved.flag("joined")); assert!(!saved.flag("zone_moved"));
    assert_eq!(saved.text("next"), "mr join");
    f.report(&["--json", "join"], "join", 0, "joined");
    let joined = status(&f, "joined");
    assert!(joined.flag("saved")); assert!(joined.flag("joined")); assert!(!joined.flag("zone_moved"));
    assert_eq!(joined.text("next"), "wait for the zone's tests");
    assert!(joined.strings("changed").is_empty()); assert!(joined.strings("untracked").is_empty());
}

#[test]
fn rit_status_unsaved_tracked_edit_and_new_cap_refusal_name_actual_bytes() {
    // mortar status -> Snapshot saved_state/cap + Gitio paths, no snapshot/save operation.
    let f = Fixture::new(); f.save_topic();
    let mut config = f.config.clone(); config.cap_bytes = 3; f.configure(&config);
    f.write("src/file", b"tracked work not yet saved\n"); f.write("src/large", b"1234567");
    let report = status(&f, "unsaved");
    assert!(!report.flag("saved")); assert_eq!(report.text("next"), "mr save \"<sentence>\"");
    assert_eq!(report.strings("changed"), &["src/file"]);
    assert_eq!(report.strings("untracked"), &["src/large"]);
    assert_eq!(report.strings("refused"), &["src/large"]); assert_eq!(report.number("cap"), 3);
    assert!(report.lines().contains("not saved: src/large is 7 bytes, over the size cap of 3 bytes"));
    assert_eq!(fs::read(f.request.join("src/large")).unwrap(), b"1234567");
}

#[test]
fn rit_status_reports_zone_moves_by_ancestry_not_the_original_cut() {
    // mortar status -> real request/zone Gitio ancestry (S-107).
    let f = Fixture::new(); f.save_topic(); f.advance_zone(false);
    let report = status(&f, "saved");
    assert!(report.flag("zone_moved")); assert!(!report.flag("joined"));
    assert_eq!(report.text("next"), "mr join");
    assert!(report.lines().contains("the zone has commits this request has not taken in"));
    f.report(&["--json", "join"], "join", 0, "rebased");
    // The rebase moved the folder to the new tip, so its tree is the tip's: saved (S-144). The zone does not hold the
    // rebased tip yet, so it is not joined.
    assert!(!status(&f, "saved").flag("zone_moved"), "rebased request has taken the zone in despite its old cut record");
}

#[test]
fn rit_status_reads_a_real_paused_join_commit_and_files_without_resolving_it() {
    // Orc#1312: actual mortar join -> durable Join pause -> actual mortar status, not fabricated PausedJoin data.
    let f = Fixture::new(); let bad = f.save_topic(); f.advance_zone(true);
    let paused = f.report(&["--json", "join"], "join", 2, "conflicted");
    assert_eq!(paused.text("commit"), bad.as_str());
    let report = status(&f, "paused");
    assert_eq!(report.text("paused_commit"), bad.as_str()); assert_eq!(report.strings("paused_files"), &["src/file"]);
    assert_eq!(report.text("next"), "mr resolve"); assert!(report.is_null("stalled_to"));
    assert_eq!(fs::read(f.request.join("src/file")).unwrap(), b"topic work\n", "status must not write markers");
}

#[test]
fn rit_status_missing_request_or_zone_ref_refuses_without_mutation() {
    // mortar status -> read_tips -> real missing refs, not synthetic report values.
    for missing in [REQUEST, ZONE] {
        let f = Fixture::new(); f.save_topic(); raw(&f.git, &["update-ref", "-d", missing]);
        let old = before(&f, &f.request);
        let report = check(f.app(&["--json", "status"]), "status", 1, "refused");
        assert_eq!(report.lines(), format!("the branch {missing} does not exist"));
        assert_eq!(report.text("owner"), if missing == ZONE { "the zone's coordinator" } else { "this request" });
        assert_eq!(before(&f, &f.request), old);
    }
}

#[test]
fn rit_status_two_real_requests_report_their_shared_path_symmetrically() {
    // mortar status -> read_overlaps -> actual records/snapshot tips -> join::overlap (two kits).
    let f = Fixture::new(); f.save_topic();
    let other = peer(&f, "peer", &["src".into(), "sub".into()]);
    write(&other, "src/file", b"other request work\n");
    check(f.command(&other, &["--json", "save", "Save peer"]).output().unwrap(), "save", 0, "saved");
    let ours = status(&f, "saved");
    assert_eq!(ours.strings("overlaps"), &["peer"]); assert_eq!(ours.strings("overlap_paths"), &["src/file"]);
    let theirs = status_at(&f, &other, "saved");
    assert_eq!(theirs.strings("overlaps"), &["task"]); assert_eq!(theirs.strings("overlap_paths"), &["src/file"]);
}

#[test]
fn rit_status_rebased_request_excludes_zone_only_changes_from_overlap() {
    // S-108: an old record.base would count src/upstream as our work and falsely overlap with peer.
    let f = Fixture::new(); f.save_topic();
    let other = peer(&f, "peer", &["src".into(), "sub".into()]);
    write(&other, "src/upstream", b"peer change to the future zone path\n");
    check(f.command(&other, &["--json", "save", "Save peer"]).output().unwrap(), "save", 0, "saved");
    let zone = f.advance_zone(false);
    f.report(&["--json", "join"], "join", 0, "rebased");
    assert_eq!(f.git.commit_info(&f.head()).unwrap().parents, vec![zone]);
    let report = status(&f, "saved");
    assert!(report.strings("overlaps").is_empty()); assert!(report.strings("overlap_paths").is_empty());
    assert!(!report.flag("zone_moved"));
    assert_eq!(report.text("next"), "mr join", "the folder's tree is the rebased tip's; the zone does not hold it yet");
}

#[test]
fn rit_status_corrupt_peer_record_fails_instead_of_claiming_no_overlap() {
    // read_overlaps must surface a real unreadable/undecodable worktree record.
    let f = Fixture::new(); f.save_topic();
    let other = peer(&f, "peer", &["src".into()]);
    let path = record_path(&f, &other); fs::write(&path, b"{not a folder record\n").unwrap();
    let old = before(&f, &f.request);
    let report = check(f.app(&["--json", "status"]), "status", 5, "failed");
    assert!(report.lines().contains(&format!("the folder record of {} cannot be read", other.display())));
    assert!(report.lines().contains(&path.display().to_string())); assert_eq!(before(&f, &f.request), old);
}

#[test]
fn rit_status_rejects_arguments_and_zone_folders_and_reads_attempt_folders_before_mutation() {
    // Actual shell/actor/attempt decisions with real folder records, not status_report ZIT data; a fresh attempt
    // (its try branch at its base, its tree the tip's) reads as saved with no work yet (S-145).
    let f = Fixture::new(); let old = before(&f, &f.request);
    check(f.app(&["--json", "status", "unexpected"]), "status", 4, "usage");
    assert_eq!(before(&f, &f.request), old);
    check(f.command(&f.zone, &["--json", "status"]).output().unwrap(), "status", 1, "refused");
    assert_eq!(before(&f, &f.request), old);
    let attempts = f.root.join("attempts");
    oplog::recover(&f.git, &f.common).unwrap();
    folders::make_attempts(&setup(&f.git, &attempts), "core", "task", &["src".into()], 1).unwrap();
    let attempt = attempts.join("try-1"); let old = before(&f, &attempt);
    check(f.command(&attempt, &["--json", "status"]).output().unwrap(), "status", 0, "saved");
    assert_eq!(before(&f, &attempt), old);
}

#[test]
fn rit_status_unstaged_edit_after_completed_rebase_is_not_a_stalled_move() {
    // S-97 index criterion: an ordinary later file edit leaves the index at the new tip.
    let f = Fixture::new(); f.save_topic(); f.advance_zone(false);
    f.report(&["--json", "join"], "join", 0, "rebased");
    f.write("src/file", b"new unstaged local work\n");
    let report = status(&f, "unsaved");
    assert!(report.is_null("stalled_from")); assert!(report.is_null("stalled_to"));
    assert_eq!(report.text("next"), "mr save \"<sentence>\"");
    assert_eq!(report.strings("changed"), &["src/file"]);
}
