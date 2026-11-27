//! The report of an `mr guard install` that land's check_hooks_path stopped, in the cli's own words: land's and
//! gitio's Display texts are never printed.
//! hooks_path_report_tests checks both arms against hand-written reports.

use land::HooksPathError;

use crate::report::{failure_report, ExitCode, Report};

const VERB: &str = "guard";

/// - Relative { value }: exit 3 Refused, outcome "refused", lines "core.hooksPath is \"<value>\", not an absolute
///   path: ..." (git resolves a relative hooks path in each folder, so one install cannot guard them all) and
///   "nothing was written", owner "the repository's git config", next unsetting core.hooksPath or setting it to an
///   absolute path, then `mr guard install --actor <name>`. Fields: none.
/// - Git: failure_report, "git could not read core.hooksPath".
pub fn hooks_path_report(error: &HooksPathError) -> Report {
    match error {
        HooksPathError::Relative { value } => Report {
            verb: VERB.to_string(),
            exit: ExitCode::Refused,
            outcome: "refused".to_string(),
            lines: vec![
                format!(
                    "core.hooksPath is \"{}\", not an absolute path: git resolves a relative hooks path in each \
                     folder, so one install cannot guard them all",
                    value.display()
                ),
                "nothing was written".to_string(),
            ],
            owner: Some("the repository's git config".to_string()),
            next: "unset core.hooksPath or set it to an absolute path, then mr guard install --actor <name>"
                .to_string(),
            fields: Vec::new(),
        },
        HooksPathError::Git(_) => failure_report(VERB, "git could not read core.hooksPath"),
    }
}
