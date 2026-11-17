//! Joins cli::run_land with land::land, folders::move_folder, gitio and oplog; shell refusals use real locks/config.
use super::fixture::Landing;
use crate::support::raw;
use oplog::{KillPidProbe, LockName, LockRecord};

#[test]
fn fast_forward_and_merge_landings_move_the_folder_when_present_and_repeat_without_another_op() {
    for merge in [false, true] {
        for with_folder in [false, true] {
            let s = Landing::new(with_folder);
            let origin = if merge {
                let origin = s.f.commit(&s.f.base, &[("src/upstream", b"upstream work\n")], "upstream\n");
                raw(&s.f.git, &["update-ref", "refs/heads/main", origin.as_str()]);
                raw(&s.f.git, &["update-ref", "refs/heads/integration", origin.as_str()]);
                if with_folder { s.sync(&s.f.base, &origin); }
                origin
            } else { s.f.base.clone() };
            let r = s.land(); let tip = s.tip();
            assert_eq!(r.text("integration"), tip.as_str());
            assert_eq!(r.text("commit"), s.squash.commit.as_str());
            if merge {
                assert_ne!(tip, s.squash.commit);
                assert_eq!(s.f.git.commit_info(&tip).unwrap().parents, vec![origin, s.squash.commit.clone()]);
                assert_eq!(s.f.blob(&tip, "src/upstream"), b"upstream work\n");
            } else { assert_eq!(tip, s.squash.commit); }
            assert_eq!(s.f.blob(&tip, "src/file"), b"zone work\n");
            if with_folder { assert_eq!(r.text("folder"), s.folder.to_str().unwrap()); s.assert_clean_at(&tip, b"zone work\n"); }
            else { assert!(r.is_null("folder")); }
            assert_eq!(s.f.records().iter().filter(|r| r.kind == "land" && r.phase == oplog::OpPhase::Done).count(), 1);
            let before = s.observe(); s.land(); assert_eq!(s.observe(), before);
        }
    }
}

#[test]
fn conflicting_landing_lists_files_and_leaves_integration_folder_log_and_state_unchanged() {
    let s = Landing::new(true);
    let origin = s.f.commit(&s.f.base, &[("src/file", b"origin work\n")], "origin\n");
    raw(&s.f.git, &["update-ref", "refs/heads/main", origin.as_str()]);
    raw(&s.f.git, &["update-ref", "refs/heads/integration", origin.as_str()]); s.sync(&s.f.base, &origin);
    let before = s.observe();
    let r = s.run("land", Some(s.squash.commit.as_str()), 2, "conflict");
    assert_eq!(r.strings("files"), &["src/file"]); assert_eq!(r.text("owner"), "the change's zone");
    assert_eq!(s.observe(), before);
}

#[test]
fn another_squash_is_busy_with_the_in_flight_change_and_a_change_id_is_usage_not_a_landing() {
    let s = Landing::new(true); let peer = s.peer_squash(); s.land();
    let before = s.observe();
    let r = s.run("land", Some(peer.commit.as_str()), 3, "busy");
    assert_eq!(r.text("busy_with"), s.squash.change_id.as_str()); assert_eq!(s.observe(), before);
    let r = s.run("land", Some(s.squash.change_id.as_str()), 4, "usage");
    let id = s.squash.change_id.as_str();
    assert_eq!(r.strings("lines")[0], format!("\"{id}\" is a change ID; land takes the squash's commit id only (find it with mr would-merge {id} --actor <name>)"));
    assert_eq!(s.observe(), before);
}

#[test]
fn live_land_lock_prevents_all_four_verbs_and_identifies_its_real_holder() {
    let s = Landing::new(true);
    let holder = LockRecord { pid: std::process::id(), host: oplog::host_name().unwrap(), key: "rit6-held-land".into(), since: 1700000000 };
    let lock = oplog::acquire_lock(&s.f.common, &LockName::Land, &holder, &KillPidProbe).unwrap();
    let before = s.observe();
    for verb in ["land", "would-merge", "back-out", "publish"] {
        let word = matches!(verb, "land" | "would-merge").then_some(s.squash.commit.as_str());
        let r = s.run(verb, word, 3, "busy");
        assert_eq!(r.number("holder_pid"), u64::from(holder.pid)); assert_eq!(r.text("holder_host"), holder.host);
        assert_eq!(r.text("holder_key"), holder.key); assert_eq!(r.number("since"), holder.since);
        assert_eq!(s.observe(), before);
    }
    oplog::release_lock(&lock).unwrap();
    s.run("would-merge", Some(s.squash.commit.as_str()), 0, "merges");
}

#[test]
fn without_config_the_new_verbs_refuse_and_point_to_init_without_mutating_the_repository() {
    let s = Landing::new(true);
    // Move this fixture's config aside; no production or team's configuration is touched.
    let config = s.f.common.join("mortar/config.json");
    std::fs::rename(&config, config.with_extension("saved")).unwrap();
    let before = s.observe();
    for verb in ["land", "would-merge"] {
        let r = s.run(verb, Some(s.squash.commit.as_str()), 1, "no-config");
        assert!(r.text("next").contains("mr init")); assert_eq!(s.observe(), before);
    }
}
