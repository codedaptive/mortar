//! CLI run_after_publish/locked_act joined to actual Oplog Land locking and shell config loading.
use super::fixture::World;
use oplog::{KillPidProbe, LockName, LockRecord};
use std::fs;

#[test]
fn a_live_land_lock_refuses_before_after_publish_and_names_the_exact_holder() {
    let w = World::new(); w.core(&[("src/file", b"published\n")]); w.publish();
    let holder = LockRecord { pid: std::process::id(), host: oplog::host_name().unwrap(), key: "rit8-held-land".into(), since: 1700000000 };
    let lock = oplog::acquire_lock(&w.s.f.common, &LockName::Land, &holder, &KillPidProbe).unwrap();
    let before = w.store(); let folders = (w.folder_state(&w.s.f.zone), w.folder_state(&w.b));
    let r = w.s.app(&["after-publish", "core", "--move", "b"], 3, "busy");
    assert_eq!(r.number("holder_pid"), u64::from(holder.pid)); assert_eq!(r.text("holder_host"), holder.host);
    assert_eq!(r.text("holder_key"), holder.key); assert_eq!(r.number("since"), holder.since);
    assert_eq!(r.text("owner"), format!("pid {} on {}", holder.pid, holder.host));
    assert_eq!(r.text("next"), "repeat the same command later");
    assert_eq!(w.store(), before); assert_eq!((w.folder_state(&w.s.f.zone), w.folder_state(&w.b)), folders);
    assert!(lock.path.exists()); oplog::release_lock(&lock).unwrap(); assert!(!lock.path.exists());
    let done = w.after(0, "after-published"); assert_eq!(done.strings("moved"), &["core", "b"]);
}

#[test]
fn missing_config_refuses_without_any_work_and_names_init_as_next() {
    let w = World::new(); w.core(&[("src/file", b"published\n")]); w.publish();
    let config = cli::config_path(&w.s.f.common); fs::rename(&config, config.with_extension("held")).unwrap();
    let before = w.store(); let folders = (w.folder_state(&w.s.f.zone), w.folder_state(&w.b));
    let r = w.after(1, "no-config");
    assert_eq!(r.strings("lines"), &[format!("there is no Mortar config at {}", config.display()), "nothing ran".into()]);
    assert_eq!(r.text("next"), "mr init"); assert_eq!(w.store(), before);
    assert_eq!((w.folder_state(&w.s.f.zone), w.folder_state(&w.b)), folders);
}
