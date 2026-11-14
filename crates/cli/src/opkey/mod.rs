//! The op key of a command: --key, or one derived from what the command is about to move.

mod op_key;

#[cfg(test)]
mod op_key_tests;

pub use op_key::op_key;
