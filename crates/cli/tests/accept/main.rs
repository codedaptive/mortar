//! The acceptance drive (`make accept`): a black-box drive of the built binary, built under both its names (`mr`
//! and `mortar`) and run as `mr`, through the mission's Done list, on throwaway repositories only, writing a
//! plain-text report. The acceptance steps are ignored, so no unit, zone or repo integration run picks them up;
//! `make accept` runs them. The unit tests of the drive's own pure pieces (no process, file, environment variable,
//! clock or repository) are not ignored, so zone runs pick them up (rule 11).

mod agent;
mod flow;
mod guard;
mod json;
mod kill;
mod report;
mod stage;
mod steps;
mod world;

/// One case for the whole drive, so the report holds every step's block in one file: each step still runs in its
/// own world, and a failing or panicking step does not stop the next.
#[test]
#[ignore = "acceptance: make accept"]
fn done_list() {
    let (path, passed) = steps::run_drive();
    assert!(passed, "the acceptance drive did not pass; the report is at {}", path.display());
}
