//! The report of a verb run where `mr init` has not run yet (S-70).

use std::path::Path;

use crate::report::{ExitCode, Report};

/// Refused (exit 1), outcome "no-config", naming the missing file; the next step is `mr init`.
pub fn missing_config_report(verb: &str, path: &Path) -> Report {
    Report {
        verb: verb.to_string(),
        exit: ExitCode::Refused,
        outcome: "no-config".to_string(),
        lines: vec![format!("there is no Mortar config at {}", path.display()), "nothing ran".to_string()],
        owner: Some("whoever sets up Mortar in this repository".to_string()),
        next: "mr init".to_string(),
        fields: Vec::new(),
    }
}
