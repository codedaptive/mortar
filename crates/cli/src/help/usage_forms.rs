//! usage_forms: a verb's usage as one line per form, for its help.

use super::split_usage_forms;
use crate::args::find_verb;

/// `mr --version` is read before the verb table (version::version_asked), so its usage is not in it.
const VERSION_USAGE: &str = "mr --version [--json]";

/// "--version": its one usage line. A verb of the table: its usage, one line per form (split_usage_forms). Any
/// other name: no lines.
pub fn usage_forms(name: &str) -> Vec<String> {
    if name == "--version" {
        return vec![VERSION_USAGE.to_string()];
    }
    find_verb(name).map_or_else(Vec::new, |verb| split_usage_forms(verb.usage))
}
