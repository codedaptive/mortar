//! Git::with_trailers: a message with trailers added. It only runs git, so it has no unit tests;
//! tests/typed/trailers.rs covers it: trailers added below a body divider that Git::trailers reads back.

use crate::read_message::read_message;
use crate::trailers_add_request::trailers_add_request;
use crate::{Git, GitError};

impl Git {
    /// `message` with `trailers` added in order under git's trailer rules (the repo's trailer.* config; by default a
    /// trailer equal to its neighbour is not added), except that a `---` line does not end the message.
    pub fn with_trailers(&self, message: &str, trailers: &[(String, String)]) -> Result<String, GitError> {
        let (args, opts) = trailers_add_request(message, trailers)?;
        read_message(&self.run(&args, &opts)?.stdout)
    }
}
