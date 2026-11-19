//! Git::trailers: a message's trailers. It only runs git, so it has no unit tests; tests/typed/trailers.rs covers
//! it: trailers in order with folded values unfolded, and none read above a body divider.

use crate::read_trailers::read_trailers;
use crate::trailers_parse_request::trailers_parse_request;
use crate::{Git, GitError};

impl Git {
    /// The trailers of `message` as (key, value) in order, by git's trailer rules, except that a `---` line does not
    /// end the message.
    pub fn trailers(&self, message: &str) -> Result<Vec<(String, String)>, GitError> {
        let (args, opts) = trailers_parse_request(message)?;
        read_trailers(&self.run(&args, &opts)?.stdout)
    }
}
