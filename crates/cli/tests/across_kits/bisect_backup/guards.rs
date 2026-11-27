//! CLI shell/locked_act joined to real Oplog land locks and config loading for bisect, backup and restore-backup.
use super::fixture::Harness;
use oplog::{KillPidProbe, LockName, LockRecord};
use std::fs;

fn words<'a>(s: &'a Harness, verb: &'a str) -> Vec<&'a str> {
    match verb {
        "bisect" => vec![verb, s.f.base.as_str(), s.f.base.as_str(), "--timeout", "60", "--", "/usr/bin/true"],
        "backup" => vec![verb],
        "restore-backup" => vec![verb, "bisect-backup-no-backup"],
        _ => unreachable!(),
    }
}

#[test]
fn all_three_verbs_refuse_a_real_live_land_lock_and_name_its_holder_without_running_the_act() {
    let s = Harness::new();
    let holder = LockRecord { pid: std::process::id(), host: oplog::host_name().unwrap(), key: "bisect-backup-live-land".into(), since: 1700000000 };
    let lock = oplog::acquire_lock(&s.f.common, &LockName::Land, &holder, &KillPidProbe).unwrap();
    let before = s.observe();
    for verb in ["bisect", "backup", "restore-backup"] {
        let r = s.app(&words(&s, verb), 4, "busy");
        assert_eq!(r.number("holder_pid"), u64::from(holder.pid)); assert_eq!(r.text("holder_host"), holder.host);
        assert_eq!(r.text("holder_key"), holder.key); assert_eq!(r.number("since"), holder.since);
        assert_eq!(r.text("owner"), format!("pid {} on {}", holder.pid, holder.host));
        assert_eq!(r.text("next"), "repeat the same command later"); assert_eq!(s.observe(), before);
    }
    oplog::release_lock(&lock).unwrap(); assert!(!lock.path.exists()); s.assert_no_probes_left();
}

#[test]
fn all_three_verbs_without_config_refuse_before_work_and_point_to_the_built_init_command() {
    let s = Harness::new(); let config = s.f.common.join("mortar/config.json");
    fs::rename(&config, config.with_extension("saved")).unwrap(); let before = s.observe();
    for verb in ["bisect", "backup", "restore-backup"] {
        let r = s.app(&words(&s, verb), 3, "no-config");
        assert_eq!(r.strings("lines"), &[format!("there is no Mortar config at {}", config.display()), "nothing ran".into()]);
        assert_eq!(r.text("next"), "mr init"); assert_eq!(s.observe(), before);
    }
    s.assert_no_probes_left();
}
