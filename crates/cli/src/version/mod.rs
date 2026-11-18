//! `mr --version` and `mortar --version` (SPEC-cli 07, "--version"): whether it is asked, and its one line.

mod version_asked;
mod version_text;

#[cfg(test)]
mod version_asked_tests;
#[cfg(test)]
mod version_text_tests;

pub(crate) use version_asked::version_asked;
pub(crate) use version_text::version_text;
