//! JoinAnswer: what a join or a resolve did. A declaration only; join_answer_display_tests exercises it.

use gitio::Oid;
use std::path::PathBuf;

/// The outcome of a join or a resolve that was not refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JoinAnswer {
    /// The request tip is already in the zone; no ref moved.
    Joined { zone_ref: String, request_tip: Oid },
    /// The zone ref moved forward from the zone tip to the request tip.
    FastForwarded { zone_ref: String, from: Oid, to: Oid },
    /// The request ref moved from its old tip to its commits replayed onto the zone tip; the zone did not
    /// move. The caller moves the request's folder to `to`; its unit tests run again before the next join.
    /// `folder_tree` is Some from a resolve that completes: the old request tip's tree with every path resolved in
    /// this join taken from the caller's tree, and the caller moves the folder from it to `to`. It is None
    /// from a plain join, and the caller moves the folder from `from`.
    Rebased { request_ref: String, from: Oid, to: Oid, folder_tree: Option<Oid> },
    /// A replayed commit did not apply; no ref moved. `files` are the conflicted paths and `state` is the
    /// mortar-join.json that `mr resolve` continues from.
    Conflicted { commit: Oid, files: Vec<PathBuf>, state: PathBuf },
    /// A resolve's first run wrote the conflicted versions of `files`, markers included, from commit `commit`'s
    /// conflict into the folder; no ref moved. `skipped` are conflicted paths the conflicted tree has no version of (a
    /// rename/rename conflict), so nothing was written for them. `gitlinks` are conflicted paths that are
    /// submodules, so no file was written for them either; their next step is to populate each one in the folder and
    /// check out the commit it should have there, then resolve again (an empty submodule folder is recorded as the
    /// request tip's commit). The next step is to edit the files written, then run `mr resolve` again.
    FilesWritten { commit: Oid, files: Vec<PathBuf>, skipped: Vec<PathBuf>, gitlinks: Vec<PathBuf> },
}
