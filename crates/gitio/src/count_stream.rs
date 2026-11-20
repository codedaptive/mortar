//! count_stream: the characters a reader yields, counted as they are read.

use crate::count_chars::count_chars;
use std::io::{self, ErrorKind, Read};

/// Bytes read per call: large enough that a big log costs few reads, small enough to live on the stack.
const CHUNK: usize = 64 * 1024;

/// The characters (see `count_chars`) read from `reader` up to end of input, one fixed buffer at a time; the input is
/// never held whole. An interrupted read is retried; any other read error is returned.
pub(crate) fn count_stream<R: Read>(mut reader: R) -> io::Result<u64> {
    let mut buffer = [0u8; CHUNK];
    let mut total = 0;
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => return Ok(total),
            Ok(read) => total += count_chars(&buffer[..read]),
            Err(err) if err.kind() == ErrorKind::Interrupted => {}
            Err(err) => return Err(err),
        }
    }
}
