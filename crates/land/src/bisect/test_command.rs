//! The command bisect runs at each probe. A bare type; tests/bisect_zit.rs exercises it
//! (bisect_names_the_merge_landing_that_broke_the_script runs check.sh through it).

use std::ffi::OsString;
use std::path::PathBuf;

/// The caller's test command: Mortar picks no tests, it runs this one in each probe's folder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TestCommand {
    /// The program to run.
    pub program: PathBuf,
    /// Its arguments, passed as given.
    pub args: Vec<OsString>,
}
