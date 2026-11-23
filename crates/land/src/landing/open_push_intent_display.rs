//! Display for OpenPushIntent: says a publish was interrupted, names the open push intent and the next command.

use crate::landing::OpenPushIntent;
use std::fmt;

impl fmt::Display for OpenPushIntent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "push intent {} is still open: a publish was interrupted; next: mr publish", self.op_id)
    }
}
