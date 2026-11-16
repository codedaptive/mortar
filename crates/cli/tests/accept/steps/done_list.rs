//! The mission's Done list, in the order the report shows it. Data only: the drive (main.rs, `done_list`) runs it.

use super::{StepSpec, AGENT, DISK, FLOW, GUARD, KILL, SHELL};

pub const DONE_LIST: [StepSpec; 6] = [SHELL, FLOW, GUARD, KILL, AGENT, DISK];
