//! Shell/CLI upkeep and snapshot runners joined to real config, folder records and Oplog lock refusal.
use super::fixture::*;
use crate::support::{raw, write};
use oplog::{KillPidProbe, LockName, LockRecord};
use std::fs;

#[test]
fn snapshot_and_sweep_respect_the_actual_join_and_folders_locks_with_exact_holder_and_no_act() {
    for (verb, name, outcome) in [("snapshot", LockName::Join("core".into()), "snapshotted"), ("sweep", LockName::Folders, "swept")] {
        let w = World::new(); w.s.f.write("src/file", b"unsaved bytes\n");
        raw(&w.s.f.git, &["update-ref", "refs/t3/eligible", w.s.f.base.as_str()]);
        write(&w.s.f.request, "target/out", b"build not removed while locked");
        let holder = LockRecord { pid: std::process::id(), host: oplog::host_name().unwrap(), key: format!("snapshot-overlap-disk-sweep-held-{verb}"), since: 1700000000 };
        let lock = oplog::acquire_lock(&w.s.f.common, &name, &holder, &KillPidProbe).unwrap(); let before = observe(&w);
        let words = if verb == "snapshot" { vec![verb] } else { vec![verb, "--ref", "refs/t3/eligible", "--idle-days", "0"] };
        let r = run(&w, &w.s.f.request, &words, 4, "busy");
        assert_eq!(r.number("holder_pid"), u64::from(holder.pid)); assert_eq!(r.text("holder_host"), holder.host);
        assert_eq!(r.text("holder_key"), holder.key); assert_eq!(r.number("since"), 1700000000);
        assert_eq!(r.text("owner"), format!("pid {} on {}", holder.pid, holder.host));
        assert_eq!(r.strings("lines"), &[format!("the lock {} is held by pid {} on {} for op {} since 1700000000 (unix seconds); nothing ran", lock.path.display(), holder.pid, holder.host, holder.key)]);
        assert_eq!(r.text("next"), "repeat the same command later"); assert_eq!(observe(&w), before);
        assert!(lock.path.exists()); oplog::release_lock(&lock).unwrap();
        let done = run(&w, &w.s.f.request, &words, 0, outcome); assert!(!lock.path.exists());
        if verb == "snapshot" {
            let id = gitio::Oid::parse(done.text("commit")).unwrap(); assert_eq!(w.s.f.blob(&id, "src/file"), b"unsaved bytes\n");
        } else { assert_eq!(ref_at(&w, "refs/t3/eligible"), None); assert!(!w.s.f.request.join("target").exists()); }
    }
}

#[test]
fn all_four_verbs_without_config_refuse_before_any_work_and_name_init() {
    let w = World::new(); w.s.f.write("src/file", b"not snapshotted\n");
    raw(&w.s.f.git, &["update-ref", "refs/t3/eligible", w.s.f.base.as_str()]);
    let config = w.s.f.common.join("mortar/config.json"); fs::rename(&config, config.with_extension("held")).unwrap();
    let before = observe(&w);
    for words in [vec!["snapshot"], vec!["overlap", "core"], vec!["disk"], vec!["sweep", "--ref", "refs/t3/eligible"]] {
        let r = run(&w, &w.s.f.request, &words, 3, "no-config");
        assert_eq!(r.strings("lines"), &[format!("there is no Mortar config at {}", config.display()), "nothing ran".into()]);
        assert_eq!(r.text("next"), "mr init"); assert_eq!(observe(&w), before);
    }
}

#[test]
fn disk_and_sweep_with_no_configured_zones_refuse_before_reading_corrupt_folders_or_deleting_refs() {
    let w = World::new(); raw(&w.s.f.git, &["update-ref", "refs/mortar/replayed/core/keep", w.s.f.base.as_str()]);
    raw(&w.s.f.git, &["update-ref", "refs/t3/eligible", w.s.f.base.as_str()]);
    write(&w.s.f.request, "target/out", b"must remain");
    let dir = w.s.f.git.git_dir(&w.b).unwrap(); fs::write(folders::record::record_path(&dir), b"corrupt record\n").unwrap();
    let mut config = w.s.f.init_args.clone(); config.zones.clear(); w.s.f.configure(&config); let before = observe(&w);
    for words in [vec!["disk"], vec!["sweep", "--ref", "refs/t3/eligible", "--idle-days", "0"]] {
        let r = app(&w, &words, 3, "refused");
        assert_eq!(r.strings("lines"), &["the config names no zone", "with no zone listed, a sweep would delete every zone's replay records (refs/mortar/replayed/); nothing was read or deleted"]);
        assert_eq!(r.text("owner"), "the repository's mortar config");
        assert_eq!(r.text("next"), "mr init --replace with every value the config holds and at least one --zone <zone>=<path> --actor <name>, then repeat the same command");
        assert_eq!(observe(&w), before);
    }
}

#[test]
fn invalid_words_are_exact_usage_errors_without_snapshot_overlap_measurement_or_sweep() {
    let w = World::new(); let before = observe(&w);
    for (words, why, next) in [
        (vec!["snapshot", "extra"], "snapshot takes no arguments", "mr snapshot --actor <name> [--key <operation key>] [--json]"),
        (vec!["disk", "extra"], "disk takes no arguments", "mr disk --actor <name> [--json]"),
        (vec!["overlap"], "overlap needs a zone", "mr overlap <zone> --actor <name> [--json]"),
        (vec!["overlap", "unknown"], "unknown zone \"unknown\": the config names b, core", "mr overlap <zone> --actor <name> [--json]"),
        (vec!["overlap", "core", "extra"], "unexpected word \"extra\" after the zone", "mr overlap <zone> --actor <name> [--json]"),
        (vec!["sweep", "--idle-days", "-1"], "--idle-days needs a whole number of days, not \"-1\"", "mr sweep [--idle-days <n>] [--ref <full ref name>]... --actor <name> [--key <operation key>] [--json]"),
    ] {
        let r = app(&w, &words, 2, "usage"); assert_eq!(r.strings("lines")[0], why); assert_eq!(r.text("next"), next);
        assert_eq!(observe(&w), before);
    }
}
