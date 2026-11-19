//! Whether a Done step runs, from its gate and whether its body is written.

use std::ffi::OsString;

use super::{Decision, Gate, StepSpec};

/// The gate first, so an unset variable is named even before the step's body is written: an Env gate whose variable
/// is unset or empty is NotRun("set <VAR> to <meaning>"). Then a step with no body is NotRun("not written yet:
/// <order>"). Otherwise Run.
pub fn decide_step(spec: &StepSpec, env: &dyn Fn(&str) -> Option<OsString>) -> Decision {
    if let Gate::Env { var, meaning } = spec.gate {
        if env(var).is_none_or(|value| value.is_empty()) {
            return Decision::NotRun(format!("set {var} to {meaning}"));
        }
    }
    match spec.body {
        Some(body) => Decision::Run(body),
        None => Decision::NotRun(format!("not written yet: {}", spec.order)),
    }
}
