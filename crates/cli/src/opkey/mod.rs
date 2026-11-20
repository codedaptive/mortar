//! The op key of a command: --key, or one derived from what the command is about to move.

mod hash_key;
mod op_key;

#[cfg(test)]
mod hash_key_tests;
#[cfg(test)]
mod op_key_tests;

pub(crate) use hash_key::hash_key;
pub(crate) use op_key::op_key;
