use std::ffi::OsString;

use super::{decide_step, Decision, Gate, StepSpec};
use crate::report::Checks;
use crate::world::World;

fn body(_: &World, _: &mut Checks) {}

const AGENT_GATE: Gate = Gate::Env { var: "ACCEPT_AGENT", meaning: "an agent command" };

fn spec(gate: Gate, written: bool) -> StepSpec {
    StepSpec { number: 4, title: "the agent test", order: "ACCEPT-4", gate, body: written.then_some(body as fn(&World, &mut Checks)) }
}

fn reason(decision: Decision) -> Option<String> {
    match decision {
        Decision::Run(_) => None,
        Decision::NotRun(reason) => Some(reason),
    }
}

fn unset(_: &str) -> Option<OsString> {
    None
}

#[test]
#[ignore = "acceptance: make accept"]
fn an_always_step_with_a_body_runs() {
    assert_eq!(reason(decide_step(&spec(Gate::Always, true), &unset)), None);
}

#[test]
#[ignore = "acceptance: make accept"]
fn a_step_with_no_body_names_its_order() {
    assert_eq!(reason(decide_step(&spec(Gate::Always, false), &unset)).as_deref(), Some("not written yet: ACCEPT-4"));
}

#[test]
#[ignore = "acceptance: make accept"]
fn an_unset_or_empty_variable_is_named_before_the_missing_body() {
    let empty = |_: &str| Some(OsString::new());
    for env in [&unset as &dyn Fn(&str) -> Option<OsString>, &empty] {
        assert_eq!(reason(decide_step(&spec(AGENT_GATE, false), env)).as_deref(), Some("set ACCEPT_AGENT to an agent command"));
        assert_eq!(reason(decide_step(&spec(AGENT_GATE, true), env)).as_deref(), Some("set ACCEPT_AGENT to an agent command"));
    }
}

#[test]
#[ignore = "acceptance: make accept"]
fn a_set_variable_lets_the_body_decide() {
    let set = |var: &str| (var == "ACCEPT_AGENT").then(|| OsString::from("claude -p"));
    assert_eq!(reason(decide_step(&spec(AGENT_GATE, true), &set)), None);
    assert_eq!(reason(decide_step(&spec(AGENT_GATE, false), &set)).as_deref(), Some("not written yet: ACCEPT-4"));
}
