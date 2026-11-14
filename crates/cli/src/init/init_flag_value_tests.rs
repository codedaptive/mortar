//! init_flag_value: the next word as the value; none or an empty one is refused naming the flag.

use super::init_flag_value;

#[test]
fn the_next_word_is_the_value_whatever_it_is() {
    assert_eq!(init_flag_value("--origin", Some(&"--remote".to_string())), Ok("--remote".to_string()));
}

#[test]
fn no_value_is_refused() {
    assert_eq!(init_flag_value("--origin", None), Err("--origin needs a value".to_string()));
}

#[test]
fn an_empty_value_is_refused() {
    assert_eq!(init_flag_value("--root-file", Some(&String::new())), Err("--root-file needs a non-empty value".to_string()));
}
