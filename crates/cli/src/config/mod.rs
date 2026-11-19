//! Mortar's per-repository settings: `<common>/mortar/config.json`, its type, its one-line JSON form,
//! and the protected list and snapshot settings built from it.

mod config;
mod config_error;
mod config_json_value;
mod config_path;
mod decode_config;
mod decode_formatter;
mod decode_zones;
mod encode_config;
mod formatter_config;
mod parse_config_json;
mod protected_refs;
mod read_config;
mod snapshot_settings;
mod snapshot_settings_for;
mod take_members;
mod take_name;
mod take_number;
mod take_strings;
mod write_config;
mod zone_config;

#[cfg(test)]
mod config_path_tests;
#[cfg(test)]
mod decode_config_tests;
#[cfg(test)]
mod decode_formatter_tests;
#[cfg(test)]
mod decode_zones_tests;
#[cfg(test)]
mod encode_config_tests;
#[cfg(test)]
mod parse_config_json_tests;
#[cfg(test)]
mod protected_refs_tests;
#[cfg(test)]
mod snapshot_settings_for_tests;
#[cfg(test)]
mod take_members_tests;
#[cfg(test)]
mod take_name_tests;
#[cfg(test)]
mod take_number_tests;
#[cfg(test)]
mod take_strings_tests;

pub(crate) use config::Config;
pub(crate) use config_error::ConfigError;
pub(crate) use config_path::config_path;
pub(crate) use decode_config::decode_config;
pub(crate) use encode_config::encode_config;
pub(crate) use formatter_config::FormatterConfig;
pub(crate) use protected_refs::protected_refs;
pub(crate) use read_config::read_config;
pub(crate) use snapshot_settings::SnapshotSettings;
pub(crate) use snapshot_settings_for::snapshot_settings_for;
pub(crate) use write_config::write_config;
pub(crate) use zone_config::ZoneConfig;

pub(crate) use config_json_value::ConfigJsonValue;
pub(crate) use decode_formatter::decode_formatter;
pub(crate) use decode_zones::decode_zones;
pub(crate) use parse_config_json::parse_config_json;
pub(crate) use take_members::take_members;
pub(crate) use take_name::take_name;
pub(crate) use take_number::take_number;
pub(crate) use take_strings::take_strings;
