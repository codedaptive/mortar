//! Step 1's checks 5 and 6: a conflict listed, handed over, resolved and joined. Real repositories and processes:
//! run_flow.rs runs it on every drive.

use std::fs;
use std::path::Path;

use super::{not_run, FlowState};
use crate::report::Checks;
use crate::stage::{agent_mr, git_out, holds_conflict_markers, stage_edit, stage_join};
use crate::world::World;

const FILE: &str = "src/a/a.txt";
const RESOLVED: &str = "a from r1\na from r2\n";

/// Check 5: r1 edits src/a/a.txt's one line and joins first (its own line), then r2 edits the same line and joins:
/// exit 1 "conflicted", the file listed, next `mr resolve`, and r2's folder still holding r2's own text, with no
/// markers, no unmerged entry and nothing uncommitted. Check 6: `mr resolve` writes the markers (exit 1
/// "files-written"); the drive writes both lines; `mr resolve` again (exit 0), then `mr join`: joined, and zone a
/// holds both edits and r2's file. False, with the later checks written as not run, when the conflict did not come
/// or the last join did not join.
pub fn flow_conflict(world: &World, checks: &mut Checks, state: &mut FlowState) -> bool {
    let first = stage_join(world, &state.r1, FILE, "a from r1\n");
    state.r1_edited = git_out(world, &world.work, &["rev-parse", "-q", "--verify", "zone/a"]);
    checks.check("5. r1 edits src/a/a.txt and runs mr join, first", first.is(0, "joined"), &first.saw);
    let join = stage_join(world, &state.r2, FILE, "a from r2\n");
    let text = read(&state.r2, FILE);
    let unmerged = git_out(world, &state.r2, &["ls-files", "-u"]);
    let status = git_out(world, &state.r2, &["status", "--porcelain"]);
    let files = join.texts("files");
    let conflicted = join.exit == Some(1) && join.outcome == "conflicted";
    let clean = text == "a from r2\n" && !holds_conflict_markers(&text) && unmerged.is_empty() && status.is_empty();
    let ok = conflicted && files == [FILE] && join.next == "mr resolve" && clean;
    let saw = format!(
        "{}; files [{}]; next: {}; r2's {FILE} {text:?}; unmerged [{unmerged}]; status [{status}]",
        join.saw,
        files.join(", "),
        join.next
    );
    checks.check("5. r2 edits the same line of src/a/a.txt, mr join in r2", ok, &saw);
    if !conflicted {
        not_run(checks, 6, "check 5 failed");
        return false;
    }
    resolve(world, checks, &state.r2)
}

/// Check 6's two lines: the first resolve, then the fix, the second resolve and the join.
fn resolve(world: &World, checks: &mut Checks, r2: &Path) -> bool {
    let first = agent_mr(world, r2, &["resolve"]);
    let marked = read(r2, FILE);
    let written = first.exit == Some(1) && first.outcome == "files-written";
    let ok = written && first.texts("files").iter().any(|file| file == FILE) && holds_conflict_markers(&marked);
    let saw = format!("{}; files [{}]; {FILE} {marked:?}", first.saw, first.texts("files").join(", "));
    checks.check("6. mr resolve in r2", ok, &saw);
    let fixed = stage_edit(r2, FILE, RESOLVED).map_or_else(|error| format!("not written: {error}"), |()| "written".into());
    let second = agent_mr(world, r2, &["resolve"]);
    let join = agent_mr(world, r2, &["join"]);
    let zone_text = git_out(world, &world.work, &["show", "zone/a:src/a/a.txt"]);
    let has_r2 = world.git(&world.work, &["cat-file", "-e", "zone/a:src/a/r2.txt"]).code == Some(0);
    let joined = join.is(0, "joined");
    let ok = fixed == "written" && second.exit == Some(0) && joined && zone_text == RESOLVED.trim_end() && has_r2;
    let saw = format!(
        "fix {fixed}; resolve: {}; join: {}; zone/a's {FILE} {zone_text:?}; zone/a holds src/a/r2.txt: {has_r2}",
        second.saw, join.saw
    );
    checks.check("6. write both lines into src/a/a.txt, mr resolve, mr join in r2", ok, &saw);
    if !joined {
        not_run(checks, 7, "check 6 failed");
    }
    joined
}

/// The file's text, or a note saying why it could not be read.
fn read(folder: &Path, file: &str) -> String {
    fs::read_to_string(folder.join(file)).unwrap_or_else(|error| format!("unreadable: {error}"))
}
