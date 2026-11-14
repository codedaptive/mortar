//! The config's JSON text back to the config.

use oplog::JsonError;

use super::{
    decode_formatter, decode_zones, parse_config_json, take_members, take_name, take_number, take_strings,
    Config, ConfigError,
};

/// The keys of the config, in the order encode_config writes them.
const KEYS: [&str; 9] = [
    "origin_branch",
    "integration_branch",
    "remote",
    "zones",
    "root_files",
    "build_dirs",
    "cap_bytes",
    "floor_bytes",
    "formatter",
];

/// The config in `text`, the strict inverse of encode_config: one JSON object with exactly its nine keys,
/// each once, in any order, whitespace allowed around tokens. Every refusal names the key (or byte) and what is wrong;
/// text that stops before the config is complete says so.
pub fn decode_config(text: &str) -> Result<Config, ConfigError> {
    config(text).map_err(|reason| ConfigError { reason })
}

fn config(text: &str) -> Result<Config, String> {
    let value = parse_config_json(text).map_err(|e| match e {
        JsonError::EndOfInput => "the text ends before the config is complete".to_string(),
        JsonError::Invalid(reason) => reason,
    })?;
    let [origin, integration, remote, zones, root_files, build_dirs, cap, floor, formatter] =
        take_members(value, KEYS, "the config")?;
    Ok(Config {
        origin_branch: take_name(origin, "origin_branch")?,
        integration_branch: take_name(integration, "integration_branch")?,
        remote: take_name(remote, "remote")?,
        zones: decode_zones(zones)?,
        root_files: take_strings(root_files, "root_files")?,
        build_dirs: take_strings(build_dirs, "build_dirs")?,
        cap_bytes: take_number(cap, "cap_bytes")?,
        floor_bytes: take_number(floor, "floor_bytes")?,
        formatter: decode_formatter(formatter)?,
    })
}
