//! marker_in_chunks: whether a byte string occurs in a stream read in chunks (the Mortar marker in a
//! candidate `mr`'s bytes). Pure: the caller does the reading and gives the chunks.

/// True as soon as `marker` occurs in the chunks taken end to end, wherever the chunk boundaries fall; false when
/// they run out first. The last `marker.len() - 1` bytes seen are carried into the next chunk's search, so a marker
/// split across any number of chunks is found. An empty marker is found at once.
pub fn marker_in_chunks<I, C>(chunks: I, marker: &[u8]) -> bool
where
    I: IntoIterator<Item = C>,
    C: AsRef<[u8]>,
{
    if marker.is_empty() {
        return true;
    }
    let keep = marker.len() - 1;
    let mut window: Vec<u8> = Vec::new();
    for chunk in chunks {
        window.extend_from_slice(chunk.as_ref());
        if window.windows(marker.len()).any(|bytes| bytes == marker) {
            return true;
        }
        let tail = window.len().saturating_sub(keep);
        window.drain(..tail);
    }
    false
}
