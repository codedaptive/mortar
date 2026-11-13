use super::Fixture;
use std::os::unix::process::ExitStatusExt;
use testkit::{run_with_kill_gate, test_child, write_git_kill_shim};

/// A real SIGKILL at a transaction boundary, held by blocking IPC rather than timing.
/// The shim delegates to real git, signals its worker before/after update-ref, and keeps
/// its pipes open until the outer test has reaped that worker, so Git::run cannot return.
pub fn kill_apply(fixture: &Fixture, stage: &str) {
    let shim = fixture.repo.path.join("git-crash-boundary.sh");
    write_git_kill_shim(&shim).unwrap();
    let mut command = test_child("crash::apply_worker").unwrap();
    command.env("MORTAR_TEST_REPO", &fixture.repo.path)
        .env("MORTAR_TEST_OLD", fixture.objects[0].as_str())
        .env("MORTAR_TEST_NEW", fixture.objects[1].as_str())
        .env("MORTAR_TEST_FOREIGN", fixture.objects[2].as_str())
        .env("MORTAR_TEST_CRASH", stage)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null");
    let output = run_with_kill_gate(&mut command, &fixture.repo.path).unwrap();
    assert_eq!(output.status.signal(), Some(9),
        "worker was not SIGKILLed at {stage}: {:?}\nstdout: {}\nstderr: {}",
        output.status, String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    let log = oplog::read_op_log(&fixture.common).unwrap();
    assert_eq!(log.records.len(), 1,
        "crash boundary must leave exactly one durable intent: {log:?}; child stdout: {}; stderr: {}",
        String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    assert_eq!(log.records[0].phase, oplog::OpPhase::Intent);
}
