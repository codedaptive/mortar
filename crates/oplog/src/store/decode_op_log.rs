//! The op log file's bytes back to its records, without their values.

use super::{decode_valued_log, plain_log, OpLog, OplogError};

/// The log decode_valued_log reads from `bytes`, each record's values dropped by plain_log; torn lines
/// and refusals are as decode_valued_log gives them.
pub fn decode_op_log(bytes: &[u8]) -> Result<OpLog, OplogError> {
    decode_valued_log(bytes).map(plain_log)
}
