//! map_steps: the replay loop: run one step per commit on a running tip and map each commit to its zone commit.

use crate::step::Step;
use crate::{JoinError, MappedCommit, MappedReplay, ReplayResult};
use gitio::Oid;

/// Runs `step(running tip, commit)` for each of `commits`, oldest first, the running tip starting at `start`. Clean
/// moves the running tip to the new commit; Dropped leaves it; either way the commit maps to the running tip after
/// it. At the first Conflicted it stops: the conflicted commit, the running tip, the commits after it, and the
/// conflicted tree and files, with only the commits before it mapped. Otherwise Clean at the running tip (`start`
/// for an empty list or when every commit dropped). An error from `step` returns at once.
pub(crate) fn map_steps(
    start: &Oid,
    commits: &[Oid],
    mut step: impl FnMut(&Oid, &Oid) -> Result<Step, JoinError>,
) -> Result<MappedReplay, JoinError> {
    let mut running = start.clone();
    let mut mapped = Vec::with_capacity(commits.len());
    for (index, commit) in commits.iter().enumerate() {
        match step(&running, commit)? {
            Step::Clean(new) => running = new,
            Step::Dropped => {}
            Step::Conflicted { tree, files } => {
                let result = ReplayResult::Conflicted {
                    commit: commit.clone(),
                    replayed_tip: running,
                    remaining: commits[index + 1..].to_vec(),
                    tree,
                    files,
                };
                return Ok(MappedReplay { result, mapped });
            }
        }
        mapped.push(MappedCommit { source: commit.clone(), replayed: running.clone() });
    }
    Ok(MappedReplay { result: ReplayResult::Clean(running), mapped })
}
