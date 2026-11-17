//! Recovery: settling each open intent from the refs' actual values, the guard hook's open-intent lookup, and
//! open_marked_intents, the walk status reads a marked log's open intents with, settling none.
//! Recovery leaves an intent marked with a remote open; close_intent closes one for the caller that knows
//! its outcome.

mod close_intent;
mod close_intent_record;
mod decide_recovery;
mod intent_close;
mod open_intent;
mod open_intent_moves;
mod open_intents;
mod open_intents_in;
mod open_marked_intents;
mod recover;
mod recoverable_intents;
mod recovery_close;
mod recovery_close_valued;
mod recovery_decision;
mod settled_or_first_error;

#[cfg(test)]
mod decide_recovery_tests;
#[cfg(test)]
mod open_intents_in_tests;
#[cfg(test)]
mod open_intents_tests;
#[cfg(test)]
mod open_marked_intents_tests;
#[cfg(test)]
mod recovery_close_tests;
#[cfg(test)]
mod recovery_close_valued_tests;
#[cfg(test)]
mod settled_or_first_error_tests;

pub use close_intent::close_intent;
pub use decide_recovery::decide_recovery;
pub use intent_close::IntentClose;
pub use open_intent::open_intent;
pub use open_intent_moves::open_intent_moves;
pub use open_intents::open_intents;
pub use open_marked_intents::open_marked_intents;
pub use recover::recover;
pub use recovery_close::recovery_close;
pub use recovery_close_valued::recovery_close_valued;
pub use recovery_decision::RecoveryDecision;
pub(crate) use close_intent_record::close_intent_record;
pub(crate) use open_intents_in::open_intents_in;
pub(crate) use recoverable_intents::recoverable_intents;
pub(crate) use settled_or_first_error::settled_or_first_error;
