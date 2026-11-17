//! MORTAR_MARKER: the fixed bytes every Mortar binary carries, so `mr guard install` can tell a copy of Mortar from
//! another program named `mr` by reading the file, never by running it (S-147). It is never printed.
//! No sibling test: a bare static with no behaviour (S-7); marker_in_chunks_tests and the ZIT
//! crates/cli/tests/file_has_marker_tests.rs cover the search for it. `#[used]` keeps it in the object file; main_entry's black_box keeps it through release and LTO.

/// The marker: NUL-bounded so it reads as no word in a program's text, with a random tail so no other program
/// carries it by chance.
#[used]
pub static MORTAR_MARKER: [u8; 37] = *b"\0mortar-binary-marker/S-147/7c1e4b9a\0";
