//! run_install: `mr install`: reads its arguments and the process's environment, then install_in.
//! In order: `--key` given is a usage report (install records no op, so a key would be ignored); then arguments
//! parse_install_args cannot read are a usage report; then HOME unset or empty is a refusal. Only then does it
//! read PATH, the project (its one git call), this binary's path and the terminal, and hand them to install_in.
//! No sibling test: it reads the process's environment and stdin; Nagatha's RITs through the binary cover it
//! (`mr install -y` with HOME a temporary folder, `--key` and `--mode mcp` exit 4, the picker on a terminal).

use std::io::{self, BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};

use gitio::Git;

use crate::args::{usage_report, UsageError};
use crate::report::Report;
use crate::shell::ShellContext;

use super::{home_unset_refusal, install_in, parse_install_args, InstallEnv, PACKAGE_PREFIXES};

/// Install's report, or a usage report (exit 4) or the HOME refusal (exit 1) before install_in. The project is the
/// git toplevel holding `cwd`, else `cwd`. The picker's question goes to stderr and its answer is one line of stdin
/// (end of input: no answer).
pub fn run_install(context: &ShellContext, cwd: &Path) -> Report {
    let usage = |reason: String| {
        usage_report(&UsageError { verb: Some("install".to_string()), reason, json: context.args.json })
    };
    if context.args.key.is_some() {
        return usage("install takes no --key".to_string());
    }
    let options = match parse_install_args(&context.args.rest) {
        Ok(options) => options,
        Err(reason) => return usage(reason),
    };
    let Some(home) = std::env::var_os("HOME").filter(|home| !home.is_empty()) else {
        return home_unset_refusal("install", "the person installing Mortar");
    };
    let env = InstallEnv {
        home: PathBuf::from(home),
        path: std::env::var_os("PATH"),
        cwd: cwd.to_path_buf(),
        project: Git::new(cwd).toplevel().unwrap_or_else(|_| cwd.to_path_buf()),
        exe: std::env::current_exe().ok(),
        prefixes: PACKAGE_PREFIXES.iter().map(PathBuf::from).collect(),
        terminal: io::stdin().is_terminal() && io::stdout().is_terminal(),
    };
    install_in(&env, &options, &mut |prompt| {
        eprint!("{prompt}");
        let _ = io::stderr().flush();
        let mut line = String::new();
        match io::stdin().lock().read_line(&mut line) {
            Ok(0) | Err(_) => None,
            Ok(_) => Some(line),
        }
    })
}
