//! Reads POSIX `df -P -k <path>` output: the bytes available on the path's volume.

/// The 4th whitespace column of the line after the header is available 1024-byte blocks; returns
/// bytes. Refuses output with no data line, a 4th column that is missing or not a number, and a
/// figure too large for u64 bytes. A filesystem name holding spaces would shift the columns; df
/// output with such a name is read wrongly or refused.
pub fn read_df_available(stdout: &[u8]) -> Result<u64, String> {
    let text = String::from_utf8_lossy(stdout);
    let data = text
        .lines()
        .nth(1)
        .filter(|line| !line.trim().is_empty())
        .ok_or_else(|| format!("df printed no data line: {text:?}"))?;
    let column = data
        .split_whitespace()
        .nth(3)
        .ok_or_else(|| format!("df data line has no available column: {data:?}"))?;
    let blocks: u64 = column
        .parse()
        .map_err(|_| format!("df available column {column:?} is not a number"))?;
    blocks
        .checked_mul(1024)
        .ok_or_else(|| format!("df available blocks {blocks} overflow a byte count"))
}
