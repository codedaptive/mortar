//! help_request: a command line that asks for help, and whether its answer is JSON (SPEC-cli 08).
//! A bare type (S-7); help_asked_tests and help_report_tests exercise it.

use super::HelpTopic;

/// `json`: `--json` came among the globals before any `--`. `topic`: what the help is about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelpRequest {
    pub json: bool,
    pub topic: HelpTopic,
}
