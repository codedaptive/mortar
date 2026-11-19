//! What install reads from the process it runs in, gathered once by run_install so the rest takes it as data.
//! A bare type; the install ZITs exercise it with a temporary HOME and fake PATH.

use std::ffi::OsString;
use std::path::PathBuf;

/// `home`: HOME. `path`: PATH as given, None when unset. `cwd`: the current directory. `project`: the git toplevel
/// holding `cwd`, or `cwd` outside a repository. `exe`: the running binary's path, None when unknown. `prefixes`:
/// the package-manager prefixes under which placement is skipped. `terminal`: stdin and stdout are both terminals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallEnv {
    pub home: PathBuf,
    pub path: Option<OsString>,
    pub cwd: PathBuf,
    pub project: PathBuf,
    pub exe: Option<PathBuf>,
    pub prefixes: Vec<PathBuf>,
    pub terminal: bool,
}
