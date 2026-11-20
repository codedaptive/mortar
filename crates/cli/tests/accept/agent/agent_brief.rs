//! The brief the drive gives the agent on stdin. A const only: run_agent's checks read
//! what it asks for (agent_zone_check), and write_agent_bin's conflicting change edits the line it names.

/// The request's task, two small edits to named files in src/a; the first changes the line the drive's
/// conflicting change also changes, so the agent's join conflicts. It names the mr verbs and `mr status` as the
/// guide and says nothing about git. "Keep both lines" is the intent check 1 reads.
pub const AGENT_BRIEF: &str = "\
You are working on a request in this folder, which is a Mortar request folder.

The task:
1. In src/a/a.txt, change its one line `a` to `a: agent`.
2. Add the file src/a/agent.txt holding the one line `agent was here`.

Use the mr verbs (mr status, mr save, mr join, mr resolve, mr need, mr undo). `mr status` tells you the next step.
If your change meets another change to the same line, keep both lines: the other one and yours.
You are done when `mr join` says your work is joined.
";
