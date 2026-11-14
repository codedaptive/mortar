//! JSON writing the report and the config share.

mod write_json_strings;

#[cfg(test)]
mod write_json_strings_tests;

pub(crate) use write_json_strings::write_json_strings;
