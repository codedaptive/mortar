//! decide_config_write: whether guard_install writes guard.json. decide_config_write_tests covers every case.

/// True when guard.json is absent (`on_disk` None) or its bytes differ from `text`; false when they are the same, so
/// a repeat install leaves the file alone.
pub(crate) fn decide_config_write(on_disk: Option<&[u8]>, text: &str) -> bool {
    on_disk != Some(text.as_bytes())
}
