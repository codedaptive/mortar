//! Step 5, disk (05-disk.md). ACCEPT-5 writes its body here and sets `body: Some(<its fn>)`; the step runs only
//! when ACCEPT_MOOT_COPY is set, which is only when Bob says.

use super::{Gate, StepSpec};

pub const DISK: StepSpec = StepSpec {
    number: 5,
    title: "disk",
    order: "ACCEPT-5",
    gate: Gate::Env { var: "ACCEPT_MOOT_COPY", meaning: "a copy of moot's repository, only when Bob says (05-disk.md)" },
    body: None,
};
