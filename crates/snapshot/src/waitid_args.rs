//! WaitidArgs: the idtype and options of one waitid(2) call. A declaration only; peek_exit_call_tests exercises it.

/// What peek_exit_call gives peek_leader_exit for this OS; the pid and the siginfo buffer are the caller's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct WaitidArgs {
    /// idtype_t, a C unsigned int on Linux and macOS.
    pub idtype: u32,
    /// The options bit set, a C int.
    pub options: i32,
}
