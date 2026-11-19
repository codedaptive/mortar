//! The message of the commit an orphan attempt is re-made at.

/// `<request>: attempt <k> re-made from its snapshot`.
pub fn remade_message(request: &str, k: u32) -> String {
    format!("{request}: attempt {k} re-made from its snapshot")
}
