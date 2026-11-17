//! Step 1's check 9: re-reviewed after an edit. Real repositories and processes: run_flow.rs runs it on every
//! drive.

use super::{not_run, rereview_files, rereview_only_edit, FlowState};
use crate::report::Checks;
use crate::stage::{harness_mr, stage_join, stage_request};
use crate::world::World;

/// r1's file (flow_requests.rs) with its second line edited: a one-line edit to a squash of some twenty lines.
const R1_EDITED: &str = "r1 line 1\nr4 edit\nr1 line 3\nr1 line 4\nr1 line 5\nr1 line 6\nr1 line 7\nr1 line 8\n";

/// A new request r4 in zone a edits line 2 of src/a/r1.txt and joins; `mr squash a` again gives the same change
/// ID and a new commit; `mr rereview <first> <second>` answers "rereviewed" and its range-diff (the report's lines,
/// each also a log line) changes src/a/r1.txt only, and in it exactly the edit: line 2 `r1 line 2` out and `r4 edit`
/// in (rereview_only_edit). False, with the later checks written as not run, when no second squash was made.
pub fn flow_rereview(world: &World, checks: &mut Checks, state: &mut FlowState) -> bool {
    let (r4, made) = stage_request(world, "a", "r4", &["src/a"]);
    state.r4 = r4;
    let join = stage_join(world, &state.r4, "src/a/r1.txt", R1_EDITED);
    let squash = harness_mr(world, &["squash", "a"]);
    let second = squash.text("commit");
    let resquashed = squash.is(0, "squashed") && !second.is_empty();
    let same_id = squash.text("change_id") == state.change_id;
    let review = harness_mr(world, &["rereview", &state.first, &second]);
    for line in &review.lines {
        checks.log(&format!("rereview: {line}"));
    }
    let files = rereview_files(&review.lines);
    let only_edit = rereview_only_edit(&review.lines, "src/a/r1.txt", "r1 line 2", "r4 edit");
    let ok = made.is(0, "made")
        && join.is(0, "joined")
        && resquashed
        && same_id
        && second != state.first
        && review.is(0, "rereviewed")
        && files == ["src/a/r1.txt"]
        && only_edit;
    let saw = format!(
        "folder r4: {}; join: {}; squash: {}; change {} (first {}), commit {second} (first {}); rereview: {}; \
         files changed [{}]; src/a/r1.txt changes exactly -+r1 line 2, ++r4 edit: {only_edit}",
        made.saw,
        join.saw,
        squash.saw,
        squash.text("change_id"),
        state.change_id,
        state.first,
        review.saw,
        files.join(", ")
    );
    let did = "9. mr folder request a r4, edit src/a/r1.txt, mr join in r4, mr squash a, mr rereview <first> <second>";
    checks.check(did, ok, &saw);
    if !resquashed {
        not_run(checks, 10, "check 9 failed");
        return false;
    }
    state.second = second;
    true
}
