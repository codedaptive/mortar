//! Step 4, the agent test (04-agent.md). Real repositories, processes and files: the drive (steps/agent.rs) runs
//! it as step 4's body on every drive with ACCEPT_AGENT set.

use std::path::{Path, PathBuf};

use super::{
    agent_conflict_check, agent_git_check, agent_log_check, agent_not_run, agent_session, agent_verbs_check,
    agent_zone_check, write_agent_bin, AgentFiles,
};
use crate::report::{report_path, Checks};
use crate::stage::{read_reply, stage_new, stage_request};
use crate::world::World;

/// The set-up (stage_new), then request folders r1 (the agent's) and rc (the drive's conflicting change) in zone a,
/// each on its own line; then the wrappers (write_agent_bin) and the agent, `sh -c "$ACCEPT_AGENT"` in r1
/// (agent_session), its transcript beside the report, named in a log line, and its exit on a line. Then the
/// conflict line and checks 1-4 in order, after one `mr log --json`. A set-up that failed writes all five as not
/// run, saying why.
pub fn run_agent(world: &World, checks: &mut Checks) {
    if stage_new(world, checks).is_none() {
        return agent_not_run(checks, "the set-up failed");
    }
    let r1 = request(world, checks, "r1");
    let rc = request(world, checks, "rc");
    let (Some(r1), Some(rc)) = (r1, rc) else {
        return agent_not_run(checks, "a request folder was not made");
    };
    let files = AgentFiles::new(&world.root);
    let env = match write_agent_bin(&files, world, &rc) {
        Ok(env) => env,
        Err(error) => {
            checks.check("set-up: wrote the mr and mortar wrappers and the git shim", false, &error.to_string());
            return agent_not_run(checks, "the wrappers were not written");
        }
    };
    let command = std::env::var_os("ACCEPT_AGENT").unwrap_or_default();
    let transcript = transcript_path();
    let ran = agent_session(world, &r1, &env, &command, &transcript);
    checks.log(&format!("the agent's transcript: {}", transcript.display()));
    let did = "ran the agent: sh -c \"$ACCEPT_AGENT\" in <root>/r1, the brief on stdin";
    checks.check(did, ran.code == Some(0), &ran.seen());
    let log = read_reply(&world.mortar(&world.work, &["log", "--json"]));
    agent_conflict_check(checks, &files, &log);
    agent_zone_check(world, checks);
    agent_log_check(checks, &log);
    agent_git_check(checks, &files);
    agent_verbs_check(checks, &files, &transcript);
}

/// `mr folder request a <request> <root>/<request> --paths src/a` on a line of its own; its folder when made.
fn request(world: &World, checks: &mut Checks, request: &str) -> Option<PathBuf> {
    let (folder, made) = stage_request(world, "a", request, &["src/a"]);
    let did = format!("set-up: mr folder request a {request} <root>/{request} --paths src/a --actor accept");
    checks.check(&did, made.is(0, "made"), &made.saw).then_some(folder)
}

/// `agent-transcript.txt` in the report's folder (report_path; the workspace is two folders above this crate, as
/// the drive finds it).
fn transcript_path() -> PathBuf {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace = manifest.ancestors().nth(2).unwrap_or(manifest);
    let env = |var: &str| std::env::var_os(var);
    report_path(env("ACCEPT_REPORT").as_deref(), env("CARGO_TARGET_DIR").as_deref(), workspace)
        .with_file_name("agent-transcript.txt")
}
