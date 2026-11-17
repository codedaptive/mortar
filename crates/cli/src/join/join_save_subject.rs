//! The subject of the commit `mr join` makes of unsaved work (S-72).

/// "<request>: work saved at join".
pub fn join_save_subject(request: &str) -> String {
    format!("{request}: work saved at join")
}
