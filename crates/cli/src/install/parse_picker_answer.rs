//! parse_picker_answer: The harnesses a person picked at install's prompt (spec 07 item 4).

use super::{HarnessSpec, HARNESSES};

/// Nothing for an empty answer, every detected one for "all", else the comma list's names (trimmed, empty items
/// ignored), each a detected one, once each, in HARNESSES order; Err names the first that is not.
pub fn parse_picker_answer(answer: &str, detected: &[&'static HarnessSpec]) -> Result<Vec<&'static HarnessSpec>, String> {
    let answer = answer.trim();
    if answer == "all" {
        return Ok(detected.to_vec());
    }
    let names: Vec<&str> = answer.split(',').map(str::trim).filter(|name| !name.is_empty()).collect();
    if let Some(stray) = names.iter().find(|name| !detected.iter().any(|harness| harness.name == **name)) {
        let listed = detected.iter().map(|harness| harness.name).collect::<Vec<_>>().join(", ");
        return Err(format!("{stray:?} is not a detected harness (detected: {listed})"));
    }
    Ok(HARNESSES.iter().filter(|harness| names.contains(&harness.name)).collect())
}
