//! The words of `mr init` after its globals, read into the config it is to write.

use std::collections::BTreeMap;

use folders::{DEFAULT_BUILD_DIR, DEFAULT_FLOOR_BYTES};
use snapshot::DEFAULT_SIZE_CAP;

use super::{init_flag_value, parse_zone_flag, InitOptions};
use crate::config::{Config, FormatterConfig, ZoneConfig};

/// --origin, --integration and --remote are required, once each. --zone, --root-file, --build-dir and
/// --formatter repeat, each value in the order given; build_dirs is [DEFAULT_BUILD_DIR] when no --build-dir is
/// given. --cap-bytes (default DEFAULT_SIZE_CAP), --floor-bytes (default DEFAULT_FLOOR_BYTES) and
/// --formatter-timeout are whole numbers, at most once each; --formatter-timeout comes with --formatter and only
/// with it. --replace at most once. Every value is the next word and is non-empty. Anything else is refused;
/// the reason names the word.
pub fn parse_init_args(rest: &[String]) -> Result<InitOptions, String> {
    let mut read = Read::default();
    let mut words = rest.iter();
    while let Some(flag) = words.next() {
        let flag = flag.as_str();
        match flag {
            "--replace" if read.replace => return Err("--replace is given twice".to_string()),
            "--replace" => read.replace = true,
            "--zone" => parse_zone_flag(&mut read.zones, &init_flag_value(flag, words.next())?)?,
            "--root-file" => read.root_files.push(init_flag_value(flag, words.next())?),
            "--build-dir" => read.build_dirs.push(init_flag_value(flag, words.next())?),
            "--formatter" => read.formatter.push(init_flag_value(flag, words.next())?),
            "--origin" | "--integration" | "--remote" | "--cap-bytes" | "--floor-bytes" | "--formatter-timeout" => {
                let value = init_flag_value(flag, words.next())?;
                if read.once.insert(flag.to_string(), value).is_some() {
                    return Err(format!("{flag} is given twice"));
                }
            }
            other => return Err(format!("unknown init argument {other:?}")),
        }
    }
    finish(read)
}

#[derive(Default)]
struct Read {
    once: BTreeMap<String, String>,
    zones: BTreeMap<String, ZoneConfig>,
    root_files: Vec<String>,
    build_dirs: Vec<String>,
    formatter: Vec<String>,
    replace: bool,
}

fn finish(mut read: Read) -> Result<InitOptions, String> {
    let mut name = |flag: &str| read.once.remove(flag).ok_or(format!("{flag} is required"));
    let (origin_branch, integration_branch, remote) = (name("--origin")?, name("--integration")?, name("--remote")?);
    let cap_bytes = number(&read.once, "--cap-bytes")?.unwrap_or(DEFAULT_SIZE_CAP);
    let floor_bytes = number(&read.once, "--floor-bytes")?.unwrap_or(DEFAULT_FLOOR_BYTES);
    let formatter = match (read.formatter.is_empty(), number(&read.once, "--formatter-timeout")?) {
        (true, None) => None,
        (true, Some(_)) => return Err("--formatter-timeout needs --formatter".to_string()),
        (false, None) => return Err("--formatter needs --formatter-timeout <seconds>".to_string()),
        (false, Some(timeout_seconds)) => Some(FormatterConfig { command: read.formatter, timeout_seconds }),
    };
    if read.build_dirs.is_empty() {
        read.build_dirs.push(DEFAULT_BUILD_DIR.to_string());
    }
    let config = Config {
        origin_branch,
        integration_branch,
        remote,
        zones: read.zones,
        root_files: read.root_files,
        build_dirs: read.build_dirs,
        cap_bytes,
        floor_bytes,
        formatter,
    };
    Ok(InitOptions { config, replace: read.replace })
}

/// The flag's value as a whole number of digits only (no sign), None when the flag was not given.
fn number(once: &BTreeMap<String, String>, flag: &str) -> Result<Option<u64>, String> {
    let Some(value) = once.get(flag) else {
        return Ok(None);
    };
    match value.bytes().all(|b| b.is_ascii_digit()).then(|| value.parse::<u64>()) {
        Some(Ok(number)) => Ok(Some(number)),
        _ => Err(format!("{flag} needs a whole number, not {value:?}")),
    }
}
