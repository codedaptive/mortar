//! help_topic: what a help request asks for (SPEC-cli 08, "The calls").
//! A bare type (S-7); help_asked_tests and help_report_tests exercise it.

use super::{HelpEntry, HelpGroup};

/// Program: the program help. Verb: one verb's help, with the group its row is in. Unknown: `mr help <word>` for a
/// word that is no verb (exit 4). TooMany: `mr help` with more than one argument (exit 4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HelpTopic {
    Program,
    Verb(&'static HelpGroup, &'static HelpEntry),
    Unknown(String),
    TooMany,
}
