//! unquote_c_style: undo the C-style quoting git puts on a path it prints one per line.

use crate::GitError;

/// A line not starting with `"` is returned as it is. A quoted line must end with `"` and hold only bytes other than
/// `\` and `"`, and the escapes git writes: `\\ \" \a \b \t \n \v \f \r` and `\ooo` (exactly three octal digits,
/// at most 0o377, one byte each, so a UTF-8 name comes back as its bytes). Anything else is refused naming the line.
pub(crate) fn unquote_c_style(line: &[u8]) -> Result<Vec<u8>, GitError> {
    let Some(rest) = line.strip_prefix(b"\"") else {
        return Ok(line.to_vec());
    };
    let refuse = || {
        GitError::refusal(
            Vec::new(),
            format!("not a C-quoted line: {:?}", String::from_utf8_lossy(line)),
        )
    };
    let mut body = rest.strip_suffix(b"\"").ok_or_else(refuse)?.iter();
    let mut out = Vec::with_capacity(rest.len());
    while let Some(&byte) = body.next() {
        match byte {
            b'"' => return Err(refuse()),
            b'\\' => out.push(escape(&mut body).ok_or_else(refuse)?),
            _ => out.push(byte),
        }
    }
    Ok(out)
}

/// The byte one escape stands for, reading what follows its backslash; None for an escape git does not write.
fn escape(body: &mut std::slice::Iter<u8>) -> Option<u8> {
    let byte = match *body.next()? {
        b'\\' => b'\\',
        b'"' => b'"',
        b'a' => 0x07,
        b'b' => 0x08,
        b't' => b'\t',
        b'n' => b'\n',
        b'v' => 0x0b,
        b'f' => 0x0c,
        b'r' => b'\r',
        first @ b'0'..=b'3' => {
            let digit = |byte: Option<&u8>| byte.filter(|b| (b'0'..=b'7').contains(*b)).map(|b| b - b'0');
            let (second, third) = (digit(body.next())?, digit(body.next())?);
            (first - b'0') * 64 + second * 8 + third
        }
        _ => return None,
    };
    Some(byte)
}
