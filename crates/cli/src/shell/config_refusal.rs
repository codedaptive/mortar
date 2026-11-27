//! The report of a verb whose config file cannot be read or decoded.

use std::path::Path;

use crate::config::ConfigError;
use crate::report::{ExitCode, Report};

/// Refused (exit 3): `error`'s reason, which names the file, then that nothing changed; the next step is to fix
/// or remove the file at `path`.
pub(crate) fn config_refusal(verb: &str, error: &ConfigError, path: &Path) -> Report {
    Report {
        verb: verb.to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![error.reason.clone(), "nothing changed".to_string()],
        owner: Some("whoever sets up Mortar in this repository".to_string()),
        next: format!("fix or remove {}, then repeat the same command", path.display()),
        fields: Vec::new(),
    }
}
