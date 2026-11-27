//! help: mr help, mr --help, mr -h and mr <verb> --help: whether a command line asks for help,
//! and the report that answers it: the program help's groups and rows, or one verb's usage one line per form, its
//! row's line and its notes, plain or as the `--json` object.

mod agent_group;
mod exit_code_help_lines;
mod find_help_entry;
mod folder_group;
mod help_asked;
mod help_entry;
mod help_entry_json;
mod help_group;
mod help_groups;
mod help_report;
mod help_request;
mod help_topic;
mod help_words;
mod landing_group;
mod program_help_lines;
mod scan_help_words;
mod setup_group;
mod split_usage_forms;
mod usage_forms;
mod verb_help_lines;
mod verb_notes;

#[cfg(test)]
mod exit_code_help_lines_tests;
#[cfg(test)]
mod find_help_entry_tests;
#[cfg(test)]
mod help_asked_tests;
#[cfg(test)]
mod help_entry_json_tests;
#[cfg(test)]
mod help_groups_tests;
#[cfg(test)]
mod help_report_tests;
#[cfg(test)]
mod program_help_lines_tests;
#[cfg(test)]
mod scan_help_words_tests;
#[cfg(test)]
mod split_usage_forms_tests;
#[cfg(test)]
mod usage_forms_tests;
#[cfg(test)]
mod verb_help_lines_tests;
#[cfg(test)]
mod verb_notes_tests;

pub(crate) use help_asked::help_asked;
pub(crate) use help_report::help_report;

use agent_group::AGENT_GROUP;
use exit_code_help_lines::exit_code_help_lines;
use find_help_entry::find_help_entry;
use folder_group::FOLDER_GROUP;
use help_entry::HelpEntry;
use help_entry_json::help_entry_json;
use help_group::HelpGroup;
use help_groups::HELP_GROUPS;
use help_request::HelpRequest;
use help_topic::HelpTopic;
use help_words::HelpWords;
use landing_group::LANDING_GROUP;
use program_help_lines::program_help_lines;
use scan_help_words::scan_help_words;
use setup_group::SETUP_GROUP;
use split_usage_forms::split_usage_forms;
use usage_forms::usage_forms;
use verb_help_lines::verb_help_lines;
use verb_notes::verb_notes;
