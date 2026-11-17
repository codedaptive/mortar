//! What `mr init` does with the config already there.
//! A bare type (S-7); decide_init_tests and init_report_tests exercise each variant.

/// Write: there is none. AlreadyDone: it holds the same values. Replace: it holds other values and --replace
/// was given. Refused: it holds other values. `keys` are the differing top-level config keys, in config order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InitDecision {
    Write,
    AlreadyDone,
    Replace { keys: Vec<String> },
    Refused { keys: Vec<String> },
}
