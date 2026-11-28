//! CLI undo shell setup/locked_act -> actual folder context, config and Oplog join lock.
use super::fixture::*;
use oplog::{KillPidProbe, LockName, LockRecord};
use std::fs;

#[test]
fn actual_live_join_lock_refuses_undo_with_the_complete_holder_report_and_no_act() {
    let w = World::new(); w.s.f.save_topic(); w.s.f.write("src/file", b"not snapshotted while locked\n");
    let holder = LockRecord { pid: std::process::id(), host: oplog::host_name().unwrap(), key: "undo-save-join-resolve-held-undo".into(), since: 1700000000 };
    let lock = oplog::acquire_lock(&w.s.f.common, &LockName::Join("core".into()), &holder, &KillPidProbe).unwrap();
    let before = observe(&w); let r = ours(&w, &["undo"], 4, "busy");
    assert_eq!(r.number("holder_pid"), u64::from(holder.pid)); assert_eq!(r.text("holder_host"), holder.host);
    assert_eq!(r.text("holder_key"), holder.key); assert_eq!(r.number("since"), holder.since);
    assert_eq!(r.text("owner"), format!("pid {} on {}", holder.pid, holder.host)); assert_eq!(r.text("next"), "repeat the same command later");
    assert_eq!(r.strings("lines"), &[format!("the lock {} is held by pid {} on {} for op {} since 1700000000 (unix seconds); nothing ran", lock.path.display(), holder.pid, holder.host, holder.key)]);
    assert_eq!(observe(&w), before); assert!(lock.path.exists()); oplog::release_lock(&lock).unwrap();
    ours(&w, &["undo"], 0, "undone"); assert_eq!(w.s.f.head(), w.s.f.base); assert!(!lock.path.exists());
}

#[test]
fn undo_bad_arguments_and_missing_config_stop_before_any_snapshot_or_ref_movement() {
    let w = World::new(); w.s.f.save_topic(); let before = observe(&w);
    let r = ours(&w, &["undo", "extra"], 2, "usage");
    assert_eq!(r.strings("lines")[0], "undo takes no arguments"); assert_eq!(r.text("next"), "mr undo [--key <key>] [--json]");
    assert_eq!(observe(&w), before);
    let config = w.s.f.common.join("mortar/config.json"); fs::rename(&config, config.with_extension("held")).unwrap();
    let r = ours(&w, &["undo"], 2, "no-config");
    assert_eq!(r.strings("lines"), &[format!("there is no Mortar config at {}", config.display()), "nothing ran".into()]);
    assert_eq!(r.text("next"), "mr init"); assert_eq!(observe(&w), before);
}
