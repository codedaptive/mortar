use crate::next_temp_index::next_temp_index;

#[test]
fn two_calls_give_different_paths() {
    let one = next_temp_index();
    let two = next_temp_index();
    assert_ne!(one.0, two.0);
}

fn serial_of(index: &crate::remove_on_drop::RemoveOnDrop) -> u64 {
    let name = index.0.file_name().and_then(|n| n.to_str()).unwrap_or_default();
    name.rsplit('-').next().and_then(|s| s.parse().ok()).unwrap_or_else(|| panic!("no serial in {name}"))
}

#[test]
fn a_later_call_takes_a_greater_serial() {
    let one = next_temp_index();
    let two = next_temp_index();
    assert!(serial_of(&two) > serial_of(&one), "{:?} then {:?}", one.0, two.0);
}

#[test]
fn path_lies_in_the_system_temp_folder() {
    let index = next_temp_index();
    assert_eq!(index.0.parent(), Some(std::env::temp_dir().as_path()));
}

#[test]
fn file_name_starts_with_mortar_index_and_this_process_id() {
    let index = next_temp_index();
    let name = index.0.file_name().and_then(|n| n.to_str()).unwrap_or_default().to_string();
    assert!(name.starts_with(&format!("mortar-index-{}-", std::process::id())), "{name}");
}

#[test]
fn no_file_is_made() {
    let index = next_temp_index();
    assert!(!index.0.exists());
}
