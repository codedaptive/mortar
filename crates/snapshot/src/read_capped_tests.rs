use crate::read_capped::read_capped;
use std::io::{self, Cursor, ErrorKind, Read};
use std::sync::Mutex;

/// Gives each step once, in order: an error of the given kind, or a run of bytes; then EOF.
struct Scripted {
    steps: Vec<Result<Vec<u8>, ErrorKind>>,
}

impl Read for Scripted {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.steps.is_empty() {
            return Ok(0);
        }
        match self.steps.remove(0) {
            Ok(bytes) => {
                buf[..bytes.len()].copy_from_slice(&bytes);
                Ok(bytes.len())
            }
            Err(kind) => Err(io::Error::from(kind)),
        }
    }
}

#[test]
fn ten_bytes_are_all_kept() {
    let kept = Mutex::new(Vec::new());
    let input = b"0123456789".to_vec();
    assert!(read_capped(Cursor::new(input), &kept).is_ok());
    assert_eq!(kept.into_inner().unwrap(), b"0123456789".to_vec());
}

#[test]
fn five_thousand_bytes_keep_the_first_4096() {
    let kept = Mutex::new(Vec::new());
    let input: Vec<u8> = (0..5000u32).map(|i| (i % 251) as u8).collect();
    assert!(read_capped(Cursor::new(input.clone()), &kept).is_ok());
    let kept = kept.into_inner().unwrap();
    assert_eq!(kept.len(), 4096);
    assert_eq!(kept[..], input[..4096]);
}

#[test]
fn bytes_already_kept_leave_room_for_96_more() {
    let kept = Mutex::new(vec![7u8; 4000]);
    assert!(read_capped(Cursor::new(vec![9u8; 200]), &kept).is_ok());
    let kept = kept.into_inner().unwrap();
    assert_eq!(kept.len(), 4096);
    assert_eq!(kept[..4000], [7u8; 4000][..]);
    assert_eq!(kept[4000..], [9u8; 96][..]);
}

#[test]
fn an_empty_reader_keeps_nothing() {
    let kept = Mutex::new(Vec::new());
    assert!(read_capped(Cursor::new(Vec::new()), &kept).is_ok());
    assert_eq!(kept.into_inner().unwrap(), Vec::<u8>::new());
}

#[test]
fn an_interrupted_read_is_retried() {
    let kept = Mutex::new(Vec::new());
    let pipe = Scripted { steps: vec![Err(ErrorKind::Interrupted), Ok(b"abc".to_vec())] };
    assert!(read_capped(pipe, &kept).is_ok());
    assert_eq!(kept.into_inner().unwrap(), b"abc".to_vec());
}

#[test]
fn another_read_error_is_returned() {
    let kept = Mutex::new(Vec::new());
    let pipe = Scripted { steps: vec![Err(ErrorKind::BrokenPipe)] };
    assert_eq!(read_capped(pipe, &kept).unwrap_err().kind(), ErrorKind::BrokenPipe);
}
