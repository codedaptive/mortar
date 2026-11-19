# Mortar Documentation

This is the map of Mortar's public documents and of the code behind them. It is organized by what a reader arrives wanting, not by who wrote what.

Mortar's documents are few on purpose. The program explains itself: every verb ends with `next:`, the one step to take (except `--version`, and the guard hook when it allows a move), and `mr help` lists every verb. When a document and the program disagree, the program is right, and the document has a bug.

## How to find what you want

**You are new and want to know what this is.**
Read the top of [`../README.md`](../README.md): what Mortar is, who it is for and what it is not. Then "A first run", which takes a scratch repository from nothing to one joined request.

**You are an AI deciding whether Mortar helps, or asked to install it.**
[`../AI_START_HERE.md`](../AI_START_HERE.md) says what to read and in what order, what not to promise, and how to install without surprising the user. [`../llms.txt`](../llms.txt) is the compact index.

**You are a coding agent working in a folder Mortar made.**
[`../AI_START_HERE.md`](../AI_START_HERE.md), "If you are an agent in a request folder": the six verbs, the rules, and a request from start to finish. In the folder itself, `mr status` gives the next step.

**You run a team of agents and want to set a repository up.**
[`../README.md`](../README.md), "Setting up a repository": `mr init` and each of its flags, the guard, and the folder verbs.

**You are writing the script that lands work.**
[`../README.md`](../README.md), "Landing" for the verbs in order, and "Output, exit codes and help" for the `--json` object and the exit codes a script reads.

**You want to know what a verb takes.**
Ask the program: `mr help <verb>` or `mr <verb> --help`. The usage lines live in one place in the code, [`../crates/cli/src/args/verbs.rs`](../crates/cli/src/args/verbs.rs) and the `*_spec.rs` files beside it.

**You want to know what Mortar writes into a repository.**
[`../README.md`](../README.md), "What Mortar keeps in the repository": the files under git's folder, the branches and the hidden refs.

**You want to know what the guard protects, and what it does not.**
[`../README.md`](../README.md), "The guard", then [`../SECURITY.md`](../SECURITY.md), which lists the limits plainly.

**You found a vulnerability.**
[`../SECURITY.md`](../SECURITY.md), "Reporting a vulnerability". Do not open a public issue.

**You want to change the code.**
[`../CONTRIBUTING.md`](../CONTRIBUTING.md): the crates and what each may depend on, the rules for a file, and the build and test commands.

**You want to know what changed.**
[`../CHANGELOG.md`](../CHANGELOG.md).

**You want to know how Mortar relates to Tessera and Temperare.**
[`../README.md`](../README.md), "With Tessera and Temperare". Each tool works alone.

## The documents

| Document | What it holds |
|---|---|
| [`../README.md`](../README.md) | What Mortar is and is not, install, a first run, every verb, output and exit codes, the guard, what is kept in the repository, status and license |
| [`../AI_START_HERE.md`](../AI_START_HERE.md) | The reading order for an AI, whether Mortar fits, the agent's rules, installing for a user, troubleshooting |
| [`../llms.txt`](../llms.txt) | The index for machine readers: documents, verbs, exit codes, paths, code map |
| [`../SECURITY.md`](../SECURITY.md) | What Mortar does to stay safe, what it does not defend against, how to report a vulnerability |
| [`../CONTRIBUTING.md`](../CONTRIBUTING.md) | Issues, the layout of the code, build and test, the rules for tests and documents |
| [`../CHANGELOG.md`](../CHANGELOG.md) | What each version does |
| [`../LICENSE`](../LICENSE) | The license. Not yet chosen |

## The code

One Cargo workspace, one program. Each crate is a folder under `crates/` with its own `src/` and `tests/`.

| Topic | Crate |
|---|---|
| Running git | [`gitio`](../crates/gitio) |
| The operation log, recovery after a kill, undo, restore, locks, change IDs | [`oplog`](../crates/oplog) |
| Zone, request and attempt folders; sparse paths; `need`; the disk report; sweeps | [`folders`](../crates/folders) |
| Saves, snapshots, the size cap, the formatter | [`snapshot`](../crates/snapshot) |
| Join, rebase, conflicts, resolve, overlap | [`join`](../crates/join) |
| Squash, would-merge, re-review, land, back-out, publish, after-publish, the guard, backup, bisect | [`land`](../crates/land) |
| The verbs, their arguments, their reports and exit codes | [`cli`](../crates/cli) |
| Throwaway repositories for tests | [`testkit`](../crates/testkit) |

Each source file holds one public symbol and opens with a comment that says what it does. To find where a verb is carried out, start at [`../crates/cli/src/shell/run.rs`](../crates/cli/src/shell/run.rs), which names the function that runs each verb, and follow its calls down.

## Conventions

- A document names a command, flag or path only as the program has it. Each one is checked against the code before the document is merged.
- Documents describe what the program does now. History is in the changelog and in git.
- Words Mortar uses in a fixed sense: a **zone** is a part of the repository one group of agents owns; a **request** is one piece of work in a zone; an **attempt** is one of several tries at a request; **origin** is the branch that holds proven code; **integration** is the branch a landing is tested on before it is published.
