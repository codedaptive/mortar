//! Recovery: settling each open intent from the refs' actual values.

mod decide_recovery;
mod open_intents;
mod recover;
mod recovery_close;
mod recovery_decision;
mod settled_or_first_error;

#[cfg(test)]
mod decide_recovery_tests;
#[cfg(test)]
mod open_intents_tests;
#[cfg(test)]
mod recovery_close_tests;
#[cfg(test)]
mod settled_or_first_error_tests;

pub use decide_recovery::decide_recovery;
pub use open_intents::open_intents;
pub use recover::recover;
pub use recovery_close::recovery_close;
pub use recovery_decision::RecoveryDecision;
pub(crate) use settled_or_first_error::settled_or_first_error;
