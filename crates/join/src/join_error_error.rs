//! Error for JoinError, so it can travel as a `dyn Error`. No unit test: it holds no code of its own;
//! join_error_display_tests cover the text it shows.

use crate::JoinError;

impl std::error::Error for JoinError {}
