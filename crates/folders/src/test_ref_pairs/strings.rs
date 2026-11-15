//! &str lists to owned strings.

pub(crate) fn strings(list: &[&str]) -> Vec<String> {
    list.iter().map(|item| item.to_string()).collect()
}
