//! The environment variable op_git sets for an op's ref transaction and for a caller's push, naming the op, so
//! the guard hook can find the op's open intent.
//! Const only, no sibling test; op_git_tests asserts its value as a literal.

/// The name of the variable that carries the op id to the guard hook.
pub const MORTAR_OP_ENV: &str = "MORTAR_OP";
