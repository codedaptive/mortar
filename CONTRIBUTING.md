# Contributing to Mortar

Thank you for your interest in Mortar. This document says how to report a problem, how the code is laid out, and how to build and test a change.

The short version: open an issue to report a problem or propose an idea, and open one before you write a large change. Mortar is licensed under the MIT License (see [`LICENSE`](LICENSE)).

---

## Issues: tell us what you know

If you have found a bug, seen a wrong answer, or know a case Mortar handles badly, open an issue. You do not need a fix in hand.

A good issue includes:

- the command you ran, with `--json` added, and all of its output;
- the exit code;
- `mortar --version` and `git --version`;
- what you expected instead.

Mortar's output is written to be pasted: every refusal says why, who owns the thing in the way, and the next step. If that text was wrong or unhelpful, that is a bug too.

For a problem with security consequences, do not open an issue. See "Reporting security issues" below.

---

## The code

Mortar is a Cargo workspace that ships one program, `mortar` (also built as `mr`). It needs a Rust toolchain with Cargo and `git` 2.46 or newer.

| Crate | What it holds | Depends on |
|---|---|---|
| `crates/gitio` | The one place that runs git | nothing |
| `crates/oplog` | The operation log, recovery, undo and restore, locks, change IDs | gitio |
| `crates/folders` | Worktrees, sparse paths, folder records, attempts, the disk report, sweeps | gitio, oplog |
| `crates/snapshot` | Saving a folder to a commit or a hidden ref; the formatter run | gitio, oplog |
| `crates/join` | Joining a request to its zone: replay, conflicts, resolve, overlap | gitio, oplog |
| `crates/land` | Squash, would-merge, re-review, land, back-out, publish, after-publish, the guard, backup, bisect | gitio, oplog, snapshot, join |
| `crates/cli` | The program: its verbs, arguments, output and exit codes | all of the above |
| `crates/testkit` | Throwaway repositories and child processes for tests. Never linked into the program | none of the others |

A crate depends only on crates above it in this table; `testkit` is a dev-dependency of the others, for their tests only. A new dependency in the other direction is a design question: raise it in an issue first. The only place that runs `git` is `gitio`; a new git call goes there.

House rules for a file:

- **One public symbol per file,** named after it. `mod.rs` and `lib.rs` only declare and re-export.
- **Files stay small:** about 150 lines.
- **Unit tests sit beside the file,** in `<name>_tests.rs`.
- **A comment says what the code does now and why.** No history, and no promise the code does not keep.
- **No new runtime dependency.** The program is built on the standard library alone.

---

## Build and test

```
make check                                  # type-check every crate and test
make test                                   # the unit tests of every crate
make test GROUP=integration                 # the integration tests of every crate
make test-one DIR=crates/join               # one crate's unit tests
make test-one DIR=crates/join GROUP=integration
make test-one DIR=crates/cli/src/status     # one module's unit tests
make test-one DIR=crates/cli/tests/across_kits  # one integration test target
make accept                                 # the acceptance drive
make script-test                            # checks scripts/test_one.sh's handling of DIR
```

`CARGO_TEST_ARGS=<name>` narrows a unit-group `test-one` run to the tests whose name matches. For `GROUP=integration` on a whole crate it narrows the `tests/` targets, but not the library's `integration::` modules: there libtest keeps the tests that match either the name or `integration::`.

There are three groups of test, chosen with `GROUP`:

| Group | What runs | What belongs in it |
|---|---|---|
| `unit` (the default) | Each crate's library tests | A test of one file's decisions, as data. No git, no files, no processes, no clock |
| `integration` | The library's `integration` modules, and every target under `tests/` except `validation` | A test that runs real git, or the built program, in a throwaway repository |
| `validation` | A crate's `tests/validation` target, built in release mode (no crate has one yet) | A drive of the whole program measured against a threshold |

`make accept` builds the program under both its names and drives it, as a black box, through whole flows on throwaway repositories: the set-up and agent verbs, a full landing, the guard, and kills in the middle of ref moves. It prints a plain-text report and leaves it at `target/accept/report.txt`. Two further steps run only when the environment variable each one names is set; the report says which were not run.

Rules for tests:

- **Every test makes its own throwaway repository and removes it.** `testkit` provides one. Never run a test, or an experiment, against a repository you care about.
- **A test proves an outcome, never a speed.** A test that fails on a busy machine is in the wrong group or is wrong.
- **Put a test in the group that matches what it reaches.** A unit test that runs git is an integration test; move it.

The repository carries a rule file, `temperare.toml`: clippy over every target with every warning an error, except `clippy::result_large_err` and `clippy::module_inception`. If you have Temperare installed, `tm check` runs it.

---

## Changing what a verb says or does

Mortar's text is part of its interface: agents act on the `next:` line and scripts read the `--json` fields and exit codes.

- A change to a verb's arguments, output, `--json` fields or exit code needs a test that shows the new behaviour, and a line in [`CHANGELOG.md`](CHANGELOG.md).
- Every refusal says why, who owns the thing in the way, and one next step.
- The last line of every verb's plain output is `next:` (except `--version`, and the guard hook when it allows a move).
- A verb that moves a ref takes its lock, lets the operation log settle anything a killed run left, then acts, and moves the ref through the operation log, naming the value it expects to replace.

## Changing a document

Every command, flag, path and claim in a document is checked against the code before it is merged. If the code does not support a sentence, the sentence is removed or corrected. The documents are listed in [`docs/README.md`](docs/README.md).

---

## Pull requests

Keep a change focused: one behaviour per pull request, with its tests. Say what the change does and why, and paste the output of the test commands you ran. `make check` and the unit and integration tests of the crates you touched should pass.

---

## Reporting security issues

**Do not report security-sensitive issues in public.**

Mortar moves the branches that hold a team's proven code. A problem disclosed in a public issue is disclosed to everyone before it can be fixed.

> **Security contact:** report privately via GitHub's [Report a vulnerability](https://github.com/codedaptive/mortar/security/advisories/new) form, or email <security@codedaptive.com>. See [`SECURITY.md`](SECURITY.md) for the full policy.

---

## A note on conduct

Engage in good faith, assume good faith in others, and keep discussion focused on the work. That covers it.
