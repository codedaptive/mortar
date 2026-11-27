//! The config a verb that needs one reads before it acts.
//! No unit test: it reads a real file; missing_config_report_tests and decode_config_tests cover what it
//! reports and decodes. The integration tests crates/cli/tests/save_join_runs/saves.rs and the guards.rs files under
//! crates/cli/tests/across_kits cover, with the mortar binary as a child process in a throwaway repository and verbs
//! that need the config (mr save, mr undo, mr after-publish and others): with no config it is refused, exit 3,
//! outcome "no-config", next "mr init"; with a config written by mr init the verb runs. Integration test to write
//! (not written here): with a config file that is not a config it is refused, exit 3, naming the file.

use std::path::Path;

use crate::config::{config_path, read_config, Config};
use crate::report::Report;

use super::{config_refusal, missing_config_report};

/// The config under `common`. None there: missing_config_report. A file that cannot be read or decoded is
/// refused, naming the file and why.
pub fn load_config(verb: &str, common: &Path) -> Result<Config, Report> {
    let path = config_path(common);
    match read_config(common) {
        Ok(Some(config)) => Ok(config),
        Ok(None) => Err(missing_config_report(verb, &path)),
        Err(error) => Err(config_refusal(verb, &error, &path)),
    }
}
