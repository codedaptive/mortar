//! Step 5, disk: its body is crate::disk::run_disk. It first refuses a copy that is a registered team
//! repository or a worktree of one (or one it cannot tell), on one failed line, measuring and writing nothing; else
//! it makes its zone and request folders from the copy, under its own scratch, and takes the five numbers on check
//! lines (each `not taken` when what it needs did not hold): a full checkout's bytes, the all-packages zone's and
//! its request folder's bytes, mr disk's store and mr snapshot's wall time. Its other numbers are log lines, which
//! never change the verdict: in each zone folder the copy's debug build runs before the request folder is made,
//! giving the build's wall time and the request folder's bytes with its build output; and a second zone holding one
//! kit, which the plan picks, gives the same numbers and build lines for it and its request folder, or one line
//! saying why it was not measured. The step runs only when ACCEPT_MOOT_COPY is set and not empty, which is only on request; once the copy and its git common dir
//! are read, it refuses on one failed line when ACCEPT_TEAMS, the folder of team manifests, is unset or
//! empty.

use super::{Gate, StepSpec};

pub const DISK: StepSpec = StepSpec {
    number: 5,
    title: "disk",
    order: "ACCEPT-5",
    gate: Gate::Env { var: "ACCEPT_MOOT_COPY", meaning: "a copy of a large repository, only on request" },
    body: Some(crate::disk::run_disk),
};
