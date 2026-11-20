//! Reads the config from the repository's git common dir.
//! No unit test: it reads a real file; decode_config_tests covers the decoding. The zone integration
//! tests are to cover: a common dir with no config reads None; a config written by write_config reads back
//! equal; a file holding text that is not a config is refused with a reason naming the file.

use std::io::ErrorKind;
use std::path::Path;

use super::{config_path, decode_config, Config, ConfigError};

/// The config at config_path(`common`); None when there is no config file. A read failure and text that is
/// not a config are refused, the reason naming the file.
pub fn read_config(common: &Path) -> Result<Option<Config>, ConfigError> {
    let path = config_path(common);
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(ConfigError { reason: format!("cannot read {}: {error}", path.display()) }),
    };
    decode_config(&text)
        .map(Some)
        .map_err(|error| ConfigError { reason: format!("{} is not a Mortar config: {}", path.display(), error.reason) })
}
