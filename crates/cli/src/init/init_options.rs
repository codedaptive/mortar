//! What `mortar init` was asked to do.
//! A bare type (S-7); parse_init_args_tests exercises it.

use crate::config::Config;

/// `config` is the config to write; `replace` is --replace: write it over a config holding other values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitOptions {
    pub config: Config,
    pub replace: bool,
}
