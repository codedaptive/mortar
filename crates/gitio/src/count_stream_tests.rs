use crate::count_stream::count_stream;
use std::collections::VecDeque;
use std::io::{self, ErrorKind, Read};

/// A reader that answers each read with the next scripted step, then end of input.
struct Scripted(VecDeque<io::Result<Vec<u8>>>);

impl Scripted {
    fn new(steps: Vec<io::Result<Vec<u8>>>) -> Scripted {
        Scripted(steps.into())
    }
}

impl Read for Scripted {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self.0.pop_front() {
            None => Ok(0),
            Some(Ok(bytes)) => {
                buf[..bytes.len()].copy_from_slice(&bytes);
                Ok(bytes.len())
            }
            Some(Err(err)) => Err(err),
        }
    }
}

/// A reader that yields one byte per read.
struct OneByte<'a>(&'a [u8]);

impl Read for OneByte<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let Some((&first, rest)) = self.0.split_first() else {
            return Ok(0);
        };
        buf[0] = first;
        self.0 = rest;
        Ok(1)
    }
}

#[test]
fn empty_input_is_zero() {
    assert_eq!(count_stream(io::empty()).unwrap(), 0);
}

#[test]
fn one_byte_per_read_counts_as_the_whole() {
    let text = "héllo ✓ 日本語";
    assert_eq!(count_stream(OneByte(text.as_bytes())).unwrap(), 11);
    assert_eq!(count_stream(text.as_bytes()).unwrap(), 11);
}

#[test]
fn a_character_split_across_reads_counts_once() {
    // 日 is e6 97 a5; the first read ends after its first byte, the second after the first byte of 本 (e6 9c ac).
    let reader = Scripted::new(vec![Ok(b"a\xe6".to_vec()), Ok(b"\x97\xa5\xe6".to_vec()), Ok(b"\x9c\xacb".to_vec())]);
    assert_eq!(count_stream(reader).unwrap(), 4);
}

#[test]
fn an_interrupted_read_is_retried() {
    let reader = Scripted::new(vec![
        Err(io::Error::from(ErrorKind::Interrupted)),
        Ok("é✓".as_bytes().to_vec()),
        Err(io::Error::from(ErrorKind::Interrupted)),
        Ok(b"xy".to_vec()),
    ]);
    assert_eq!(count_stream(reader).unwrap(), 4);
}

#[test]
fn another_read_error_is_returned() {
    let reader = Scripted::new(vec![Ok(b"ab".to_vec()), Err(io::Error::other("boom")), Ok(b"cd".to_vec())]);
    let err = count_stream(reader).unwrap_err();
    assert_eq!(err.kind(), ErrorKind::Other);
    assert_eq!(err.to_string(), "boom");
}

#[test]
fn input_past_one_buffer_is_counted_whole() {
    // 18 characters in 28 bytes; 5000 copies are 140000 bytes, past two 64 KiB reads. The first read ends inside a ✓:
    // 65536 = 2340 * 28 + 16, and the ✓ is bytes 14 to 16 of each copy.
    let input = "héllo wörld ✓ 日本語\n".repeat(5000);
    assert_eq!(input.len(), 140_000);
    assert_eq!(count_stream(input.as_bytes()).unwrap(), 90_000);
}
