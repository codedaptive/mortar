use super::support::*;
use join::JoinRunError;
use oplog::{HeldLockError, KillPidProbe, LockName, OplogError};
use std::{fs, os::unix::fs::PermissionsExt, process::Command};

#[test]
fn racing_zone_move_is_protected_before_logging() {
    let s = Scenario::new(false);
    let common = s.f.repo.path.join(".git");
    let root = s.f.git.commit_info(&s.zone).unwrap().parents[0].clone();
    let sideways = s.f.commit(&[("sideways", b"race\n")], Some(&root), "sideways\n");
    let before = oplog::read_valued_log(&common).unwrap().records;

    let real_git = Command::new("sh").args(["-c", "command -v git"]).output().unwrap();
    assert!(real_git.status.success());
    let real_git = String::from_utf8(real_git.stdout).unwrap();
    let wrapper = common.join("racing-git");
    let marker = common.join("racing-git-fired");
    fs::write(&wrapper, "#!/bin/sh\n\
        if [ \"$3\" = merge-base ] && [ \"$4\" = --is-ancestor ] && [ ! -e \"$RACE_MARKER\" ]; then\n\
            : > \"$RACE_MARKER\"\n\
            \"$REAL_GIT\" -C \"$2\" update-ref refs/heads/zone/zit-zone \"$RACE_TIP\" || exit $?\n\
        fi\n\
        exec \"$REAL_GIT\" \"$@\"\n").unwrap();
    fs::set_permissions(&wrapper, fs::Permissions::from_mode(0o755)).unwrap();
    let mut git = s.f.git.clone();
    git.program = wrapper;
    git.env.extend([
        ("REAL_GIT".into(), real_git.trim().into()),
        ("RACE_MARKER".into(), marker.into_os_string()),
        ("RACE_TIP".into(), sideways.as_str().into()),
    ]);

    let answer = oplog::with_lock(&git, &common, &LockName::Join(ZONE.into()), ACTOR, &KillPidProbe, || {
        oplog::recover(&git, &common).map_err(JoinRunError::Oplog)?;
        join::join_on_base(&git, &s.f.repo.path, ZONE, REQUEST_REF, &s.base, ACTOR)
    });
    let after = oplog::read_valued_log(&common).unwrap().records;
    assert!(matches!(&answer, Err(HeldLockError::Work(JoinRunError::Oplog(OplogError::Protected { .. })))),
        "expected Protected, got {answer:?}; log records: {after:?}");
    assert_eq!(after.len(), before.len());
    assert!(after.iter().all(|entry| entry.record.kind != "join-zone-request"));
    assert_eq!(s.f.git.read_ref(ZONE_REF).unwrap(), Some(sideways));
    assert_eq!(s.f.git.read_ref(REQUEST_REF).unwrap(), Some(s.request));
}
