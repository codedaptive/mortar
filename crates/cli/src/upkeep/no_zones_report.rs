//! The refusal of `mr disk` or `mr sweep` when the config names no zone, before the verb runs git.

use crate::report::{ExitCode, Report};

/// Exit 1 under `verb`: the config names no zone, and an empty zone list would have a sweep delete every zone's
/// replay records; nothing was read or deleted. The next mends the config with mr init, then repeats.
pub fn no_zones_report(verb: &str) -> Report {
    Report {
        verb: verb.to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![
            "the config names no zone".to_string(),
            "with no zone listed, a sweep would delete every zone's replay records (refs/mortar/replayed/); nothing \
             was read or deleted"
                .to_string(),
        ],
        owner: Some("the repository's mortar config".to_string()),
        next: "mr init --replace with every value the config holds and at least one --zone <zone>=<path> --actor \
               <name>, then repeat the same command"
            .to_string(),
        fields: Vec::new(),
    }
}
