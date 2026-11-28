//! The report of a verb whose config file cannot be read or decoded.

use std::path::Path;

use crate::config::ConfigError;
use crate::report::{ExitCode, Report};

/// Usage (exit 2: a configuration problem), outcome "bad-config": `error`'s reason, which names the file, then
/// that nothing changed; the next step is to fix or remove the file at `path`.
pub(crate) fn config_refusal(verb: &str, error: &ConfigError, path: &Path) -> Report {
    Report {
        verb: verb.to_string(),
        exit: ExitCode::Usage,
        outcome: "bad-config".to_string(),
        lines: vec![error.reason.clone(), "nothing changed".to_string()],
        owner: Some("whoever sets up Mortar in this repository".to_string()),
        next: format!("fix or remove {}, then repeat the same command", path.display()),
        fields: Vec::new(),
    }
}
