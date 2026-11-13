use super::Fixture;
use std::fs;
use std::os::unix::{fs::PermissionsExt, process::ExitStatusExt};
use std::process::Command;

/// A real SIGKILL at a deterministic transaction boundary, never a clock or sleep.
/// The shim executes real git unchanged, except that it kills its apply parent before
/// update-ref starts, or after update-ref succeeds but before Git::run can return.
pub fn kill_apply(fixture: &Fixture, stage: &str) {
    let shim = fixture.repo.path.join("git-crash-boundary.sh");
    fs::write(&shim, b"#!/bin/sh\nset -eu\nif [ \"${3-}\" != update-ref ]; then exec git \"$@\"; fi\n[ \"$PPID\" = \"$MORTAR_TEST_WORKER_PID\" ] || exit 98\ncase \"$MORTAR_TEST_CRASH\" in\n  before) kill -KILL \"$MORTAR_TEST_WORKER_PID\" ;;\n  after) git \"$@\"; kill -KILL \"$MORTAR_TEST_WORKER_PID\" ;;\n  *) exit 99 ;;\nesac\n").unwrap();
    fs::set_permissions(&shim, fs::Permissions::from_mode(0o700)).unwrap();
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "crash::apply_worker", "--ignored", "--nocapture"])
        .env("MORTAR_TEST_REPO", &fixture.repo.path)
        .env("MORTAR_TEST_OLD", fixture.objects[0].as_str())
        .env("MORTAR_TEST_NEW", fixture.objects[1].as_str())
        .env("MORTAR_TEST_FOREIGN", fixture.objects[2].as_str())
        .env("MORTAR_TEST_CRASH", stage)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .output().unwrap();
    assert_eq!(output.status.signal(), Some(9),
        "worker was not SIGKILLed at {stage}: {:?}\nstdout: {}\nstderr: {}",
        output.status, String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    let log = oplog::read_op_log(&fixture.common).unwrap();
    assert_eq!(log.records.len(), 1,
        "crash boundary must leave exactly one durable intent: {log:?}; child stdout: {}; stderr: {}",
        String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    assert_eq!(log.records[0].phase, oplog::OpPhase::Intent);
}
