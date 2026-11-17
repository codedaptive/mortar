//! marker_in_chunks: whether the marker occurs in the chunks read end to end, wherever the chunk boundaries fall.
//! Pure: each case is a list of chunks and the answer, written out by hand.

use super::marker_in_chunks;

const MARKER: &[u8] = b"<mortar>";

fn found(chunks: &[&[u8]]) -> bool {
    marker_in_chunks(chunks.iter().copied(), MARKER)
}

#[test]
fn the_marker_inside_one_chunk_is_found() {
    assert!(found(&[b"abc<mortar>def"]));
}

#[test]
fn the_marker_split_across_two_chunks_is_found() {
    assert!(found(&[b"abc<mor", b"tar>def"]));
}

#[test]
fn the_marker_split_across_three_chunks_is_found() {
    assert!(found(&[b"abc<m", b"ort", b"ar>def"]));
}

#[test]
fn the_marker_in_one_byte_chunks_is_found() {
    let bytes = b"xx<mortar>yy";
    assert!(marker_in_chunks(bytes.chunks(1), MARKER));
}

#[test]
fn the_marker_split_one_byte_before_a_boundary_is_found() {
    assert!(found(&[b"abc<mortar", b">def"]));
    assert!(found(&[b"abc<", b"mortar>def"]));
}

#[test]
fn an_absent_marker_is_not_found() {
    assert!(!found(&[b"abc<mort", b"ar def", b"<mortar"]));
}

#[test]
fn only_a_prefix_of_the_marker_at_the_end_is_not_found() {
    assert!(!found(&[b"abcdef", b"<mortar"]));
}

#[test]
fn empty_input_is_not_found() {
    assert!(!found(&[]));
    assert!(!found(&[b"", b""]));
}

#[test]
fn a_partial_match_does_not_hide_a_full_one_after_it() {
    assert!(found(&[b"<mor<mo", b"rtar>"]));
}
