//! Git::trailers: a message's trailers. It only runs git, so it has no unit tests; Nagatha's integration test
//! "trailers reading a message's trailers in order with folded values unfolded" is to cover it.

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
