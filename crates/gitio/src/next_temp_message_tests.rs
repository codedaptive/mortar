use crate::next_temp_message::next_temp_message;

fn serial_of(message: &crate::remove_on_drop::RemoveOnDrop) -> u64 {
    let name = message.0.file_name().and_then(|n| n.to_str()).unwrap_or_default();
    name.rsplit('-').next().and_then(|s| s.parse().ok()).unwrap_or_else(|| panic!("no serial in {name}"))
}

#[test]
fn a_later_call_takes_a_greater_serial() {
    let one = next_temp_message();
    let two = next_temp_message();
    assert!(serial_of(&two) > serial_of(&one), "{:?} then {:?}", one.0, two.0);
}

#[test]
fn path_lies_in_the_system_temp_folder() {
    let message = next_temp_message();
    assert_eq!(message.0.parent(), Some(std::env::temp_dir().as_path()));
}

#[test]
fn file_name_starts_with_mortar_message_and_this_process_id() {
    let message = next_temp_message();
    let name = message.0.file_name().and_then(|n| n.to_str()).unwrap_or_default().to_string();
    assert!(name.starts_with(&format!("mortar-message-{}-", std::process::id())), "{name}");
}

#[test]
fn no_file_is_made() {
    let message = next_temp_message();
    assert!(!message.0.exists());
}
