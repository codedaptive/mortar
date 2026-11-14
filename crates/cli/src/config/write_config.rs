//! Writes the config into the repository's git common dir.
//! No unit test: it writes real files (S-5); encode_config_tests covers the text. Nagatha's integration group
//! is to cover: write then read_config reads back equal; a second write replaces the first whole, through
//! config.json.tmp renamed over config.json, leaving no tmp file; a common dir with no `mortar` folder gets
//! one.

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use super::{config_path, encode_config, Config, ConfigError};

/// Creates `<common>/mortar` when absent, writes encode_config(`config`) to config.json.tmp, syncs it and
/// renames it over config.json, so a reader sees the old config or the new one, never part of one. Every
/// failure names the file or folder it was working on.
pub fn write_config(common: &Path, config: &Config) -> Result<(), ConfigError> {
    let path = config_path(common);
    let tmp = path.with_extension("json.tmp");
    let failed = |what: &Path, error: std::io::Error| ConfigError {
        reason: format!("cannot write {}: {error}", what.display()),
    };
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| failed(dir, e))?;
    }
    let mut file = File::create(&tmp).map_err(|e| failed(&tmp, e))?;
    file.write_all(encode_config(config).as_bytes()).map_err(|e| failed(&tmp, e))?;
    file.sync_all().map_err(|e| failed(&tmp, e))?;
    fs::rename(&tmp, &path).map_err(|e| failed(&path, e))
}
