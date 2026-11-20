//! Step 1's checks 1-3: requests made, saved and joined. Real repositories and processes: run_flow.rs runs it on
//! every drive.

use std::path::Path;

use super::{files_in, not_run, sparse_strays, trailers_missing, FlowState};
use crate::report::Checks;
use crate::stage::{agent_mr, git_out, stage_request, stage_save};
use crate::world::World;

/// Eight lines, so the squash's patch outweighs check 9's one-line edit and git's range-diff pairs the two squashes.
const R1_TEXT: &str = "r1 line 1\nr1 line 2\nr1 line 3\nr1 line 4\nr1 line 5\nr1 line 6\nr1 line 7\nr1 line 8\n";

/// Check 1 makes r1 and r2 in zone a and r3 in zone b, one line each; check 2 saves a new file in r1; check 3 joins
/// r1. False, with the later checks written as not run, when a folder was not made, the save did not save, or the
/// join did not join.
pub fn flow_requests(world: &World, checks: &mut Checks, state: &mut FlowState) -> bool {
    let mut made = true;
    for (zone, request, path) in [("a", "r1", "src/a"), ("a", "r2", "src/a"), ("b", "r3", "src/b")] {
        made &= request_made(world, checks, zone, request, path);
    }
    (state.r1, state.r2, state.r3) = (world.root.join("r1"), world.root.join("r2"), world.root.join("r3"));
    if !made {
        not_run(checks, 2, "check 1 failed");
        return false;
    }
    let save = stage_save(world, &state.r1, "src/a/r1.txt", R1_TEXT, "r1: first");
    let count = git_out(world, &world.work, &["rev-list", "--count", "main..req/a/r1"]);
    let trailers = git_out(world, &world.work, &["log", "-1", "--format=%(trailers:only,unfold)", "req/a/r1"]);
    let missing = trailers_missing(&trailers, "r1", "a");
    let saw = format!(
        "{}; req/a/r1 is {count} commit(s) past main; trailers [{}]; missing [{}]",
        save.saw,
        trailers.lines().collect::<Vec<_>>().join(", "),
        missing.join(", ")
    );
    let saved = save.is(0, "saved");
    checks.check("2. write src/a/r1.txt in r1, mr save \"r1: first\"", saved && count == "1" && missing.is_empty(), &saw);
    if !saved {
        not_run(checks, 3, "check 2 failed");
        return false;
    }
    let join = agent_mr(world, &state.r1, &["join"]);
    let zone = git_out(world, &world.work, &["rev-parse", "-q", "--verify", "zone/a"]);
    let tip = git_out(world, &world.work, &["rev-parse", "-q", "--verify", "req/a/r1"]);
    let joined = join.is(0, "joined");
    let ok = joined && join.next == "wait for the zone's tests" && zone == tip && join.text("to") == tip;
    let saw = format!("{}; next: {}; zone/a {zone}, req/a/r1 {tip}, to {}", join.saw, join.next, join.text("to"));
    checks.check("3. mr join in r1", ok, &saw);
    if !joined {
        not_run(checks, 4, "check 3 failed");
    }
    joined
}

/// `mr folder request <zone> <request> <root>/<request> --paths <path>`: made, and the folder holds files of
/// `path` and nothing but those and the root file Cargo.toml.
fn request_made(world: &World, checks: &mut Checks, zone: &str, request: &str, path: &str) -> bool {
    let (folder, reply) = stage_request(world, zone, request, &[path]);
    let did = format!("1. mr folder request {zone} {request} <root>/{request} --paths {path} --actor accept");
    let (held, listed) = held_files(&folder, path);
    checks.check(&did, reply.is(0, "made") && held, &format!("{}; {listed}", reply.saw))
}

/// Whether the folder holds at least one file of `path` and no stray, and the text that says what it holds.
fn held_files(folder: &Path, path: &str) -> (bool, String) {
    let files = match files_in(folder) {
        Ok(files) => files,
        Err(error) => return (false, format!("the folder could not be listed: {error}")),
    };
    let strays = sparse_strays(&files, &[path], &["Cargo.toml"]);
    let own = files.iter().any(|file| file.starts_with(&format!("{path}/")));
    (own && strays.is_empty(), format!("files [{}]; strays [{}]", files.join(", "), strays.join(", ")))
}
