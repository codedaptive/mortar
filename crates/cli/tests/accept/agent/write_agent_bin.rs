//! Writes the agent's `mr` and `mortar` wrappers and its logging git shim, and the environment that puts them on
//! its PATH. Real files: run_agent.rs writes them on every drive with ACCEPT_AGENT set.
//!
//! The wrappers are thin on purpose: each logs one line and runs the real program, so what the agent sees is
//! Mortar's and git's own behaviour. The one addition is in the `mr` wrapper: on the agent's first `mr join`, before
//! that join runs, it makes the drive's conflicting change. The change lands after every save
//! the agent ran before that join and before the join itself (which saves first), so the join conflicts whether or
//! not the agent ran `mr save`, with no timer. That is why the wrappers come first on the agent's PATH (04-agent.md puts the build's folder
//! first): the agent's `mr` must be the wrapper. Each line is tab-separated: pid, parent pid, the parent's command
//! base name, then the argv.

use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use testkit::write_git_shim;

use super::AgentFiles;
use crate::world::World;

/// Every script's start: the parent's command base name (`ps` may print a full path; a space becomes `_`).
const PARENT: &str = r#"#!/bin/sh
tab=$(printf '\t')
parent=$(ps -o comm= -p "$PPID" | tr ' ' '_')
parent=${parent##*/}
"#;

/// In the wrappers, a call made under the drive's conflicting change is the drive's.
const DRIVE: &str = r#"[ -n "$ACCEPT_DRIVE" ] && parent=drive
"#;

/// The `mr` wrapper's conflict, before it logs the agent's call: once, on the agent's first `mr join`, it writes
/// the drive's line in rc, then runs `mr save` and `mr join` there with the built mr, each logged as the drive's,
/// their output in ACCEPT_DRIVE_OUT. The agent's call is logged after them, so mr.log's order is the order they ran.
const MR_CONFLICT: &str = r#"verb=
for arg in "$@"; do case $arg in -*) ;; *) verb=$arg; break ;; esac; done
drive() {
  line="$$$tab$$${tab}drive${tab}mr"
  for arg in "$@"; do line="$line$tab$arg"; done
  printf '%s\n' "$line" >> "$ACCEPT_MR_LOG"
  (cd "$ACCEPT_RC" && ACCEPT_DRIVE=1 "$ACCEPT_REAL_MR" "$@") >> "$ACCEPT_DRIVE_OUT" 2>&1
}
if [ "$verb" = join ] && [ -z "$ACCEPT_DRIVE" ] && [ ! -e "$ACCEPT_ONCE" ]; then
  : > "$ACCEPT_ONCE"
  printf '%s\n' "$ACCEPT_RC_TEXT" > "$ACCEPT_RC/$ACCEPT_RC_FILE"
  drive save "rc: conflict"
  drive join
fi
"#;

/// The `mr` wrapper's end: runs the built mr as a child and exits with its code.
const MR_RUN: &str = r#""$ACCEPT_REAL_MR" "$@"
exit $?
"#;

/// The file and line the drive's conflicting change writes in rc: the line AGENT_BRIEF's first edit changes.
const RC_FILE: &str = "src/a/a.txt";
const RC_TEXT: &str = "a: drive";

/// Writes `<bin>/mr`, `<bin>/mortar` and `<shim>/git`, and returns the agent's PATH (bin, shim, then the world's
/// PATH) and what the scripts read: the built mr and mortar, the real git (the first `git` on the world's PATH),
/// the logs, the once-marker, rc's folder and the line it writes. An error when a file cannot be written or no git
/// is on the world's PATH.
pub fn write_agent_bin(files: &AgentFiles, world: &World, rc: &Path) -> io::Result<Vec<(OsString, OsString)>> {
    let git = std::env::split_paths(&world.path)
        .map(|folder| folder.join("git"))
        .find(|path| path.is_file())
        .ok_or_else(|| io::Error::other("no git on the world's PATH"))?;
    fs::create_dir_all(&files.bin)?;
    fs::create_dir_all(&files.shim)?;
    let mr = format!("{PARENT}{DRIVE}{MR_CONFLICT}{}{MR_RUN}", log_line("mr", "ACCEPT_MR_LOG"));
    let mortar = format!("{PARENT}{DRIVE}{}exec \"$ACCEPT_REAL_MORTAR\" \"$@\"\n", log_line("mortar", "ACCEPT_MR_LOG"));
    let shim = format!("{PARENT}{}exec \"$ACCEPT_REAL_GIT\" \"$@\"\n", log_line("git", "ACCEPT_GIT_LOG"));
    write_git_shim(&files.bin.join("mr"), mr.as_bytes())?;
    write_git_shim(&files.bin.join("mortar"), mortar.as_bytes())?;
    write_git_shim(&files.shim.join("git"), shim.as_bytes())?;
    let front = [files.bin.clone(), files.shim.clone()];
    let path = std::env::join_paths(front.into_iter().chain(std::env::split_paths(&world.path)))
        .map_err(io::Error::other)?;
    let paths: [(&str, PathBuf); 9] = [
        ("ACCEPT_REAL_MR", World::mr_bin().to_path_buf()),
        ("ACCEPT_REAL_MORTAR", World::mortar_bin().to_path_buf()),
        ("ACCEPT_REAL_GIT", git),
        ("ACCEPT_MR_LOG", files.mr_log.clone()),
        ("ACCEPT_GIT_LOG", files.git_log.clone()),
        ("ACCEPT_ONCE", files.once.clone()),
        ("ACCEPT_DRIVE_OUT", files.drive_out.clone()),
        ("ACCEPT_RC", rc.to_path_buf()),
        ("ACCEPT_RC_FILE", PathBuf::from(RC_FILE)),
    ];
    let mut env = vec![(OsString::from("PATH"), path), (OsString::from("ACCEPT_RC_TEXT"), OsString::from(RC_TEXT))];
    env.extend(paths.into_iter().map(|(key, value)| (OsString::from(key), value.into_os_string())));
    Ok(env)
}

/// The lines that append "<pid> <parent pid> <parent> <name> <args...>", tab-separated, to the file `$<log>`.
fn log_line(name: &str, log: &str) -> String {
    format!(
        "line=\"$$$tab$PPID$tab$parent${{tab}}{name}\"\nfor arg in \"$@\"; do line=\"$line$tab$arg\"; done\n\
         printf '%s\\n' \"$line\" >> \"${log}\"\n"
    )
}
