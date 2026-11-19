//! help_request: a command line that asks for help, and whether its answer is JSON.
//! A bare type; help_asked_tests and help_report_tests exercise it.

use super::HelpTopic;

/// `json`: `--json` came among the globals before any `--`. `topic`: what the help is about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelpRequest {
    pub json: bool,
    pub topic: HelpTopic,
}
