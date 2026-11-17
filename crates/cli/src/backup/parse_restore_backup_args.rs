//! The words of `mortar restore-backup` after its globals, read into the host whose backup to restore.

/// Exactly one word, the host, taken as it is (land's restore_backup refuses a host that cannot name a namespace).
/// The Err is the reason: no word, or a second word (named).
pub fn parse_restore_backup_args(rest: &[String]) -> Result<String, String> {
    let Some((host, words)) = rest.split_first() else {
        return Err("restore-backup needs the host whose backup to restore".to_string());
    };
    if let Some(extra) = words.first() {
        return Err(format!("unexpected word {extra:?} after the host"));
    }
    Ok(host.clone())
}
