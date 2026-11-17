//! 03's case 4: a publish killed between its push and its local move. Real repositories, processes and a SIGKILL:
//! run_kill.rs runs it on every drive.

use std::fs;

use super::{common_dir, done_moves_of, kill_armed, kill_consistent, read_landing, read_log, CaseLines, KillState, OpLine};
use crate::report::Checks;
use crate::stage::{agent_mr, git_out, harness_mr, remote_git};
use crate::world::{Crash, World};

const MAIN: &str = "refs/heads/main";

/// With `squash` landed on integration and not yet published: `mr publish --actor accept --json` killed before its
/// first update-ref, which comes after its push (advance_origin's move of the local main): the open intent is a
/// publish-origin. The landing state (`<common>/mortar/landing.json`) still names the squash, landing it at
/// integration's tip from the local main, while the remote's main is already there: a publish interrupted; what
/// `mr status` in r3 says is a log line. `mr publish` again exits 0 "published"; the remote's and the local main
/// are at the landed commit, the remote one commit on, the local main moved by one done op since the kill. The push
/// intent (publish-push, to the landed commit) was closed done in the log the kill left, before any recover ran, and
/// is the only one. Then the consistency items, repeating the publish.
pub fn kill_publish(world: &World, checks: &mut Checks, state: &KillState, squash: &str) {
    let mut lines = CaseLines::new(4);
    let local = || git_out(world, &world.work, &["rev-parse", "-q", "--verify", MAIN]);
    let (main, landed) = (local(), git_out(world, &world.work, &["rev-parse", "-q", "--verify", "integration"]));
    let args = ["publish", "--actor", "accept", "--json"];
    match kill_armed(world, checks, &mut lines, (&world.work, &args), Crash::Before, "publish-origin") {
        None => lines.not_run(checks, "the kill did not stop a publish at its local move", false),
        Some((killed, at_kill)) => {
            let (ok, saw) = interrupted(world, squash, &main, &landed);
            lines.check(checks, ok, &saw);
            let status = agent_mr(world, &state.r3, &["status"]);
            let said = format!("next: {}; lines {:?}", status.next, status.lines);
            checks.log(&format!("mr status in r3 after the kill: {}; {said}", status.saw));
            let publish = harness_mr(world, &["publish"]);
            lines.check(checks, publish.is(0, "published"), &publish.saw);
            let (remote, now) = (remote_git(world, &["rev-parse", "main"]), local());
            let count = remote_git(world, &["rev-list", "--count", &format!("{main}..main")]);
            let moves = read_log(world).map(|log| done_moves_of(&log, MAIN, &killed.op_id).len());
            let ok = remote == landed && now == landed && count == "1" && moves == Ok(1);
            let saw = format!(
                "remote main {remote}, local main {now}, landed {landed}; remote commits on: {count}; done local \
                 moves since the kill: {moves:?}"
            );
            lines.check(checks, ok, &saw);
            push_closed(world, checks, &mut lines, &at_kill, &landed);
        }
    }
    kill_consistent(world, checks, &mut lines, &state.folders, &|| harness_mr(world, &["publish"]));
}

/// The landing state names `squash`, landing at `landed` from `main`, the local main is still at `main` and the
/// remote's main at `landed`: the push went through and the local move did not.
fn interrupted(world: &World, squash: &str, main: &str, landed: &str) -> (bool, String) {
    let path = common_dir(world).join("mortar").join("landing.json");
    let seen = fs::read_to_string(&path).map_err(|error| error.to_string()).and_then(|text| read_landing(&text));
    let (remote, local) = (remote_git(world, &["rev-parse", "main"]), git_out(world, &world.work, &["rev-parse", MAIN]));
    let named = seen.as_ref().is_ok_and(|seen| {
        seen.squash == squash && seen.integration == landed && seen.origin_at_start == main
    });
    let saw = format!("landing.json {seen:?}; remote main {remote}, local main {local}, landed {landed}");
    (named && remote == landed && local == main, saw)
}

/// The push intent's line: in the log the kill left, a done publish-push to `landed`; in the log now, one
/// publish-push intent to it and one done record. Each such record is a log line.
fn push_closed(world: &World, checks: &mut Checks, lines: &mut CaseLines, at_kill: &[OpLine], landed: &str) {
    let push = |op: &&OpLine| op.kind == "publish-push" && op.moves.iter().any(|(_, new)| new == landed);
    let done_at_kill = at_kill.iter().filter(push).any(|op| op.phase == "done");
    let log = match read_log(world) {
        Ok(log) => log,
        Err(error) => {
            lines.check(checks, false, &error);
            return;
        }
    };
    let pushes: Vec<&OpLine> = log.iter().filter(push).collect();
    for op in &pushes {
        checks.log(&format!("the push: {}", op.shown()));
    }
    let count = |phase: &str| pushes.iter().filter(|op| op.phase == phase).count();
    let (intents, dones) = (count("intent"), count("done"));
    let saw = format!(
        "closed done before the repeat (in the log the kill left): {done_at_kill}; publish-push intents {intents}, \
         done {dones}"
    );
    lines.check(checks, done_at_kill && intents == 1 && dones == 1, &saw);
}
