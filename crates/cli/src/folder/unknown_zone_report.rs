//! The refusal of an `mr folder` sub-verb whose zone the config does not name: zone and request name it on their
//! line, refresh reads it from the zone folder's record.

use crate::config::Config;
use crate::report::{ExitCode, Report};

/// Exit 3 "refused": the zone is not in the config's zones (those it has are named), nothing was made; the next step
/// is the harness's own: init's whole command with --replace, the zone added to what the config already holds.
pub fn unknown_zone_report(zone: &str, config: &Config) -> Report {
    let known: Vec<&str> = config.zones.keys().map(String::as_str).collect();
    Report {
        verb: "folder".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![
            format!("zone {zone} is not in the Mortar config, whose zones are: {}", known.join(", ")),
            "nothing was made".to_string(),
        ],
        owner: Some("the zones in the Mortar config".to_string()),
        next: format!(
            "mr init --replace --actor <name> --origin <branch> --integration <branch> --remote <name> \
             --zone {zone}=<path>[,<path>...] with every other zone and setting the config already holds, then repeat"
        ),
        fields: Vec::new(),
    }
}
