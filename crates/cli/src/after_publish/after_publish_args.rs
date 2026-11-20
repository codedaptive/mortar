//! What `mr after-publish` reads from its line.
//! A bare type; parse_after_publish_args_tests and repeat_command_tests exercise it.

/// The zone whose squash was published, and each zone named by --move, in the order given.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AfterPublishArgs {
    pub published: String,
    pub moves: Vec<String>,
}
