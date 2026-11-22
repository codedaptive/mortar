//! withdraw: a zone's squash withdrawn before it lands, its squash ref deleted in one op, with the refusals for a
//! squash already published or in the landing in flight.

mod decide_withdraw;
// The module is named after the function it holds, which shares the folder's name.
#[allow(clippy::module_inception)]
mod withdraw;
mod withdraw_answer;
mod withdraw_error;
mod withdraw_error_display;
mod withdraw_error_source;
mod withdraw_op;

#[cfg(test)]
mod decide_withdraw_tests;
#[cfg(test)]
mod withdraw_error_display_tests;
#[cfg(test)]
mod withdraw_error_source_tests;

pub use withdraw::withdraw;
pub use withdraw_answer::WithdrawAnswer;
pub use withdraw_error::WithdrawError;

pub(crate) use decide_withdraw::decide_withdraw;
pub(crate) use withdraw_op::withdraw_op;
