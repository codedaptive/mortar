//! Reading `kill -0` results: only "no such process" means dead.

use super::kill_status_alive;

#[test]
fn success_is_alive() {
    assert!(kill_status_alive(true, ""));
}

#[test]
fn no_such_process_is_dead_in_any_case() {
    assert!(!kill_status_alive(false, "kill: 4242: No such process\n"));
    assert!(!kill_status_alive(false, "bash: kill: (4242) - NO SUCH PROCESS"));
}

#[test]
fn not_permitted_is_alive() {
    assert!(kill_status_alive(false, "kill: 1: Operation not permitted\n"));
}

#[test]
fn unknown_failure_is_alive() {
    assert!(kill_status_alive(false, ""));
    assert!(kill_status_alive(false, "kill: illegal process id: x"));
}
