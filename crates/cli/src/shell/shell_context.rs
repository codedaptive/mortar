//! What every verb runs with, once the shell has read it.
//! A bare type (S-7). Shell API: run builds it; the verbs of WO-cli-2..6 read it. run has no unit test, so
//! Nagatha's integration cases listed there exercise it.

use oplog::ProtectedRef;

use crate::args::ParsedArgs;
use crate::config::Config;
use crate::context::FolderContext;

/// `actor` is who the verb acts as (actor_for). `folder` is Some for agent verbs; `config` is Some for verbs
/// that need it, and `protected` is then built from it (empty otherwise).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellContext {
    pub args: ParsedArgs,
    pub actor: String,
    pub folder: Option<FolderContext>,
    pub config: Option<Config>,
    pub protected: Vec<ProtectedRef>,
}
