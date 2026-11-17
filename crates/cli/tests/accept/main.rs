//! The acceptance drive (`make accept`): a black-box drive of the built binary, built under both its names (`mr`
//! and `mortar`) and run as `mr`, through the mission's Done list, on throwaway repositories only, writing a
//! plain-text report. Every case in this target is ignored, so no unit, zone or repo integration run picks it up;
//! `cargo test -p cli --test accept -- --ignored --test-threads=1` runs it.

mod json;
mod report;
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
