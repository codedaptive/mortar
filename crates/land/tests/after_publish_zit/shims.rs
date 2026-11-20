//! The after_publish ZITs' git shims, scripts in the fixture repository's .git that run the real git: runner answers
//! git's replay probe as installed or as forced unavailable, and tracer
//! logs every git argument list after_publish runs. Each keeps the fixture git's environment (config isolated, the
//! fixed identity and dates).

use crate::fixture::Fixture;
use gitio::Git;
use std::path::{Path, PathBuf};
use testkit::write_git_shim;

/// Separates one logged argument from the next: a byte no git argument in these cases holds.
const SEPARATOR: char = '\u{1f}';

/// The fixture's git with git's replay probe (`git replay -h`) answered as unavailable when `fallback`, else real
/// git as installed. None, with a note, when `fallback` is false and this git has no native replay: that arm cannot
/// run here, and the forced-fallback arm still does.
pub fn runner(fx: &Fixture, fallback: bool) -> Option<Git> {
    if !fallback && !fx.git.replay_available().unwrap() {
        eprintln!("git's native replay is unavailable here; only the forced-fallback arm runs");
        return None;
    }
    let shim = fx.common.join("replay-path.sh");
    // Only the capability probe is overridden; every other run, native replay included, is the real git.
    let script = b"#!/bin/sh\n\
        if [ \"${3-}\" = replay ] && [ \"${4-}\" = -h ] && [ \"$MORTAR_FORCE_FALLBACK\" = 1 ]; then\n\
        printf 'forced unavailable replay\\n' >&2\n\
        exit 129\n\
        fi\n\
        exec git \"$@\"\n";
    write_git_shim(&shim, script).unwrap();
    let mut git = fx.git.clone();
    git.program = shim;
    git.env.push(("MORTAR_FORCE_FALLBACK".into(), if fallback { "1" } else { "0" }.into()));
    assert_eq!(git.replay_available().unwrap(), !fallback, "the shim must answer the replay probe as asked");
    Some(git)
}

/// The fixture's git with every run's arguments (gitio's `-C <repo>` first) appended, one run per line, to the log
/// it answers with, a file in the fixture repository's .git.
pub fn tracer(fx: &Fixture) -> (Git, PathBuf) {
    let shim = fx.common.join("trace-git.sh");
    let log = fx.common.join("git-calls");
    let script = b"#!/bin/sh\n\
        printf '%s\\037' \"$@\" >>\"$MORTAR_GIT_TRACE\"\n\
        printf '\\n' >>\"$MORTAR_GIT_TRACE\"\n\
        exec git \"$@\"\n";
    write_git_shim(&shim, script).unwrap();
    let mut git = fx.git.clone();
    git.program = shim;
    git.env.push(("MORTAR_GIT_TRACE".into(), log.clone().into_os_string()));
    (git, log)
}

/// The argument lists tracer logged, one per run in run order; empty when nothing ran.
pub fn traced_calls(log: &Path) -> Vec<Vec<String>> {
    let text = std::fs::read_to_string(log).unwrap_or_default();
    let call = |line: &str| line.split_terminator(SEPARATOR).map(str::to_string).collect();
    text.lines().map(call).collect()
}
