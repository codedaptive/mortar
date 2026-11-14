use super::{kill_worker, Fixture};

/// kill_worker with crash::apply_worker, which runs apply on operation().
pub fn kill_apply(fixture: &Fixture, stage: &str) {
    kill_worker(fixture, stage, "crash::apply_worker");
}
