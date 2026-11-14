//! The config a verb that needs one reads before it acts (S-70).
//! No unit test: it reads a real file (S-5); missing_config_report_tests and decode_config_tests cover what it
//! reports and decodes. Nagatha's integration group is to cover, with the mortar binary as a child process in a
//! throwaway repository and the first verb that needs the config (mortar join): with no config it is refused,
//! exit 1, outcome "no-config", next "mortar init"; with a config file that is not a config it is refused,
//! exit 1, naming the file; with a config written by mortar init the verb runs.

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
