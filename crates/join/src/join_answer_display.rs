//! Display for JoinAnswer: what a join did and what to do next. join_answer_display_tests covers each variant.

use crate::JoinAnswer;
use std::fmt;

impl fmt::Display for JoinAnswer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JoinAnswer::Joined { zone_ref, request_tip } => {
                write!(f, "joined: the request tip {request_tip} is already in {zone_ref}; nothing moved")
            }
            JoinAnswer::FastForwarded { zone_ref, from, to } => {
                write!(f, "joined: {zone_ref} moved forward from {from} to {to}")
            }
            JoinAnswer::Rebased { request_ref, from, to } => write!(
                f,
                "rebased: {request_ref} moved from {from} to {to}, replayed onto the zone; the unit tests must \
                 run again, then run `mortar join`"
            ),
            JoinAnswer::Conflicted { commit, files, state } => {
                write!(f, "conflicted: commit {commit} did not apply; state in {}", state.display())?;
                for file in files {
                    write!(f, "\n  {}", file.display())?;
                }
                write!(f, "\nresolve the files, then run `mortar resolve`")
            }
        }
    }
}
