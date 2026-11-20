//! The op log file's bytes back to its records, each with its values.

use super::marked::{decode_marked_log, unmarked_log};
use super::{OplogError, ValuedLog};

/// decode_marked_log(bytes) with each record's remote mark dropped: lines, torn lines and refusals as it
/// gives them. An intent whose closing record tore stays open: recover settles it from the refs, unless it
/// is marked, and recover leaves a marked intent open.
pub(crate) fn decode_valued_log(bytes: &[u8]) -> Result<ValuedLog, OplogError> {
    decode_marked_log(bytes).map(unmarked_log)
}
