//! How a bisect report reads: the answer on the first line, then one indented line per probe in the order run.

use std::fmt;

use super::{BisectAnswer, BisectNotFound, BisectReport, Culprit};

impl fmt::Display for BisectReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.answer {
            BisectAnswer::Culprit(culprit) => write_culprit(f, culprit)?,
            BisectAnswer::NotFound(BisectNotFound::FailedPasses) => {
                f.write_str("no culprit: the failed commit passes the command")?
            }
            BisectAnswer::NotFound(BisectNotFound::LastPassFails) => {
                f.write_str("no culprit: the last passing commit fails the command")?
            }
        }
        self.probes.iter().try_for_each(|probe| write!(f, "\n  {probe}"))
    }
}

/// The culprit line: its commit, then its change ID and zone, naming whichever trailer it does not carry.
fn write_culprit(f: &mut fmt::Formatter<'_>, culprit: &Culprit) -> fmt::Result {
    let commit = &culprit.commit;
    match (&culprit.change_id, &culprit.zone) {
        (Some(id), Some(zone)) => write!(f, "culprit: {commit}, change {id}, zone {zone}"),
        (Some(id), None) => write!(f, "culprit: {commit}, change {id}; it carries no Mortar-Zone trailer"),
        (None, Some(zone)) => write!(f, "culprit: {commit}, zone {zone}; it carries no Mortar-Change-Id trailer"),
        (None, None) => write!(f, "culprit: {commit}; it carries no squash trailers"),
    }
}
