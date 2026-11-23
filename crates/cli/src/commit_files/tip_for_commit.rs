//! tip_for_commit: zone/<zone>'s tip for `mr commit-files` in a zone folder, once the folder is found to stand on it.
//! No sibling test: it runs git and reads the op log and the marker file; zone_off_tip_refusal_tests and
//! stopped_run_refusal_tests cover its decisions, and crates/cli/tests/folder_runs/run_commit_files_tests.rs covers it
//! through the binary.

use std::path::{Path, PathBuf};

use gitio::{Git, IndexTreeError, Oid};
use oplog::read_op_log;

use super::read_pending_commit::read_pending_commit;
use super::{stopped_run_refusal, zone_off_tip_refusal};
use crate::context::FolderContext;
use crate::report::{failure_report, Report};
use crate::zone_folder_move::{find_zone_old, zone_values, ZoneOld};

const VERB: &str = "commit-files";

/// zone/<zone>'s tip, once the folder is found to stand on it, and whether the index's tree is the tip's. The
/// folder's index tree (unmerged entries or git failing: failure_report), the tip's tree and the op log's values of
/// the zone branch give find_zone_old's answer. AtTip goes on; From is zone_off_tip_refusal's. NoMatch with a marker
/// at `marker` that counts (read_pending_commit) goes on when the marker's tip is the tip and is stopped_run_refusal's
/// when the branch has moved since; without one it is zone_off_tip_refusal's. No commit, ref or index entry is
/// written; reading the index's tree may write tree objects.
pub(super) fn tip_for_commit(
    git: &Git,
    folder: &FolderContext,
    marker: &Path,
    paths: &[PathBuf],
) -> Result<(Oid, bool), Report> {
    let (zone, top) = (folder.record.zone.as_str(), folder.toplevel.as_path());
    let failed = |what: String| failure_report(VERB, &format!("{what}; no commit was written and no ref moved"));
    let zone_ref = format!("refs/heads/zone/{zone}");
    let tip = match git.read_ref(&zone_ref) {
        Ok(Some(tip)) => tip,
        Ok(None) => return Err(failed(format!("{zone_ref} does not exist"))),
        Err(_) => return Err(failed(format!("git could not read {zone_ref}"))),
    };
    let shown = top.display();
    let index_tree = git.index_tree(top).map_err(|error| match error {
        IndexTreeError::Unmerged { .. } => failed(format!("the zone folder {shown}'s index holds unmerged paths")),
        IndexTreeError::Git(_) => failed(format!("git could not read the zone folder {shown}'s index")),
    })?;
    let tip_tree = git.commit_info(&tip).map_err(|_| failed(format!("git could not read commit {tip}")))?.tree;
    let records = read_op_log(&folder.common).map_err(|_| failed("the op log could not be read".to_string()))?.records;
    let tree_of = |value: &Oid| git.commit_info(value).ok().map(|info| info.tree);
    let old = find_zone_old(&index_tree, &tip_tree, &zone_values(&records, &zone_ref, &tip), tree_of);
    let staged_on = (old == ZoneOld::NoMatch).then(|| read_pending_commit(git, marker, &index_tree, paths)).flatten();
    match (staged_on, zone_off_tip_refusal(zone, top, &tip, &old)) {
        (Some(staged_on), _) if staged_on == tip => Ok((tip, index_tree == tip_tree)),
        (Some(staged_on), _) => Err(stopped_run_refusal(zone, top, &staged_on, &tip, paths)),
        (None, Some(refusal)) => Err(refusal),
        (None, None) => Ok((tip, index_tree == tip_tree)),
    }
}
