//! One harness install can wire Mortar's skill into (spec 07 items 4 and 5).
//! A bare type (S-7); harnesses_tests and the planning tests exercise it.

/// `name`: the harness's name on the command line. `folder`: the folder in HOME whose presence detects it.
/// `skills`: the skills folder, relative to HOME (`--location global`) or the project (`--location local`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarnessSpec {
    pub name: &'static str,
    pub folder: &'static str,
    pub skills: &'static str,
}
