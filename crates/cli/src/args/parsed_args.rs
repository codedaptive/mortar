//! A command line once read.
//! A bare type; parse_args_tests exercises it.

use super::VerbSpec;

/// `rest` holds every word that is not the verb or a global flag, in order, the words after `--` included
/// as they are. `dash_dash` is the index in `rest` where the words after the `--` that ends the globals start; None
/// when no `--` ends them (a `--` taken as the value of `--key` or `--actor` does not). `key` and `actor` are the raw
/// values of `--key` and `--actor`; each verb decides their meaning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedArgs {
    pub verb: &'static VerbSpec,
    pub rest: Vec<String>,
    pub dash_dash: Option<usize>,
    pub json: bool,
    pub key: Option<String>,
    pub actor: Option<String>,
}
