//! The snapshot settings a save and a join use, from the config's formatter, build_dirs and cap_bytes.

use std::ffi::OsString;
use std::path::PathBuf;
use std::time::Duration;

use snapshot::Formatter;

use super::{Config, FormatterConfig, SnapshotSettings};

/// Each build dir becomes the two pathspecs snapshot's built-in excludes use: "<d>/" for the top-level directory
/// and "*/<d>/*" for nested ones. Trailing '/' is trimmed, an empty name is skipped, and a pathspec already in
/// the list is not added again.
pub fn snapshot_settings_for(config: &Config) -> SnapshotSettings {
    let mut excludes: Vec<String> = Vec::new();
    for dir in &config.build_dirs {
        let name = dir.trim_end_matches('/');
        if name.is_empty() {
            continue;
        }
        for pathspec in [format!("{name}/"), format!("*/{name}/*")] {
            if !excludes.contains(&pathspec) {
                excludes.push(pathspec);
            }
        }
    }
    SnapshotSettings {
        formatter: config.formatter.as_ref().and_then(formatter_for),
        excludes,
        cap_bytes: config.cap_bytes,
    }
}

/// decode_formatter refuses an empty command, so None here is only a guard for a Config built in code.
fn formatter_for(formatter: &FormatterConfig) -> Option<Formatter> {
    let (program, args) = formatter.command.split_first()?;
    Some(Formatter {
        program: PathBuf::from(program),
        args: args.iter().map(OsString::from).collect(),
        timeout: Duration::from_secs(formatter.timeout_seconds),
    })
}
