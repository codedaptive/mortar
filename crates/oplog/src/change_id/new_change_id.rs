//! A new change ID from 16 bytes of /dev/urandom; ChangeId::from_bytes does the encoding.
//! No unit test: it reads a real device. Integration tests should cover:
//! "new_change_id twice gives two different IDs" and "new_change_id gives 32 chars of k..z".

use std::fs::File;
use std::io::Read;

use super::{ChangeId, ChangeIdError};

/// A fresh random change ID, or Random when /dev/urandom cannot be opened or read.
pub fn new_change_id() -> Result<ChangeId, ChangeIdError> {
    let mut bytes = [0u8; 16];
    File::open("/dev/urandom")
        .and_then(|mut device| device.read_exact(&mut bytes))
        .map_err(|source| ChangeIdError::Random { source })?;
    Ok(ChangeId::from_bytes(bytes))
}
