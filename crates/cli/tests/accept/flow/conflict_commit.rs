//! A squash-shaped commit made with plain git to conflict with integration (check 8's second half). Real
//! processes: flow_land.rs makes it on every drive.

use crate::stage::git_out;
use crate::world::World;

/// A change ID no squash of the drive draws (32 letters k..z), so the commit names a change of its own.
const CHANGE_ID: &str = "kkkkllllmmmmnnnnooooppppqqqqrrrr";

/// A commit on origin's `main` with the tree of `tree_of`, carrying what would-merge reads off a squash: a subject
/// `a: 1 requests`, one request line, and the Mortar-Change-Id, Mortar-Zone and Mortar-Squash-Of trailers. Written
/// with `git commit-tree`, which moves no ref. Its id, or "" when git failed.
pub fn conflict_commit(world: &World, tree_of: &str) -> String {
    let tree = format!("{tree_of}^{{tree}}");
    let trailers = format!("Mortar-Change-Id: {CHANGE_ID}\nMortar-Zone: a\nMortar-Squash-Of: {}", world.base);
    let args = [
        "commit-tree",
        tree.as_str(),
        "-p",
        "main",
        "-m",
        "a: 1 requests",
        "-m",
        "- conflict: made to conflict with integration",
        "-m",
        trailers.as_str(),
    ];
    git_out(world, &world.work, &args)
}
