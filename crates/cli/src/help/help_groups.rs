//! help_groups: the program help's four groups in the order they print.
//! A bare const; help_groups_tests covers it.

use super::{HelpGroup, AGENT_GROUP, FOLDER_GROUP, LANDING_GROUP, SETUP_GROUP};

pub const HELP_GROUPS: &[HelpGroup] = &[AGENT_GROUP, FOLDER_GROUP, LANDING_GROUP, SETUP_GROUP];
