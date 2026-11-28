# AI Start Here: Mortar

You are an AI reading the Mortar repository for the first time. This page says what Mortar is, whether it helps you, and what to read next. Use it before improvising.

## What Mortar is

Mortar writes version-control history for teams of coding agents, with plain git. It makes a folder (a git worktree) for each piece of work, saves the work as commits, joins it to the part of the repository it belongs to, and gives the caller the verbs that land finished work on the main branch.

The short version:

- The repository stays plain git. Mortar runs `git` as a program and keeps no store of its own.
- An agent uses six verbs and never types a git write command.
- A harness, or a person, uses the other verbs to make folders and to land work.
- Every verb ends with `next:`, the one step to take (except `--version`, and the guard hook when it allows a move).
- The command is `mr`. `mortar` is the long name for the same program.

Two words you need. A **zone** is a part of the repository one group of agents owns. A **request** is one piece of work in a zone; it has its own folder and its own branch.

## Which reader are you

| You are | Go to |
|---|---|
| A coding agent working in a folder Mortar made | "If you are an agent in a request folder", below |
| Asked "would this help us?" or "install this" | "Does Mortar fit" and "Installing for a user", below |
| Building a harness or script on top of Mortar | [`README.md`](README.md): "Setting up a repository", "Landing", "Output, exit codes and help" |
| Changing Mortar's own code | [`CONTRIBUTING.md`](CONTRIBUTING.md) |

## Reading order

1. [`README.md`](README.md): what Mortar is and is not, install, a first run, every verb.
2. This page.
3. `mr help`, then `mr help <verb>`: the program's own list of verbs and each one's usage. When a document and the program disagree, the program is right.
4. [`docs/README.md`](docs/README.md): where each topic is documented, and where each part of the code lives.
5. [`SECURITY.md`](SECURITY.md): what the guard does and does not protect.
6. [`CONTRIBUTING.md`](CONTRIBUTING.md): only when changing the code.

The compact index for machine readers is [`llms.txt`](llms.txt).

## Does Mortar fit

It fits when:

- several agents work in one git repository at once and their work must not collide;
- each agent should see only the paths its work needs;
- finished work should reach the main branch one landing at a time, with a record of who moved what;
- the repository must stay readable and usable with ordinary git tools.

It does not fit when:

- the machine runs Windows: Mortar is Unix only;
- you want something to run tests, review code or decide what may land: Mortar does none of these, the caller does;
- you want a hosting, pull-request or CI tool;
- you need a security boundary between agents: the guard stops accidents, not a determined bypass.

## What not to promise

Do not tell a user that Mortar:

- runs or picks tests (`mr bisect` runs the one command the caller hands it, nothing more);
- reviews code or approves a landing;
- starts, schedules or messages agents;
- replaces git, or hides the repository from git;
- is safe against a hostile agent or a hostile user on the same machine;
- has a stable interface yet: this is version 0.4.0.

## If you are an agent in a request folder

You work in a request folder: a git worktree Mortar made for one request in one zone. `mr status` names your request and zone.

| Verb | What it does |
|---|---|
| `mr status` | Where you are, what you changed, whether your zone moved, any conflict, and the next step |
| `mr save "<sentence>"` | Keeps your work as a commit. The sentence is one line saying what the work does |
| `mr join` | Joins your request to the zone. Unsaved work is saved first |
| `mr resolve` | Writes the conflicted files into your folder with markers. Fix them, then run it again |
| `mr need <path>` | Adds a path to your folder when the work needs a file outside it |
| `mr undo` | Takes back your own last save, or the last rebase or resolve step of a join |

Rules:

- **Never type a git write command:** commit, merge, rebase, reset, checkout, switch, push, pull, stash, branch, tag, update-ref. Reading with git (status, diff, log, show) is fine.
- **Do what `next:` says.** When unsure, run `mr status`.
- **The six verbs are yours. The others are not.** `mr folder`, `mr squash`, `mr land`, `mr publish` and the rest belong to the harness that runs you.
- **Your verbs take no folder or name argument and no `--actor`.** The folder you are in says who you are.
- **A refusal is information, not an error to work around.** It says why, who owns the thing in the way, and what to do.

A request, start to finish:

1. `mr status`.
2. Make your change, then `mr save "<what it does>"`.
3. `mr join`.
   - **Joined** (exit 0, `next: wait for the zone's tests`): your part is done.
   - **Rebased** (exit 0, `next: mr join`): the zone had moved and your work now sits on top of it. Run your tests again, then `mr join` again.
   - **Conflicted** (exit 1, `next: mr resolve`): the files are listed and your folder is as it was. Run `mr resolve`; it writes each conflicted file into your folder with markers (`<<<<<<<`, `=======`, `>>>>>>>`), and lists any path it cannot write that way, such as a submodule. Fix each file, remove every marker, and run `mr resolve` again. It goes on rebasing your work and says `next: mr join`, or stops again with `next: mr resolve` if a later commit conflicts.

Exit codes: 0 done (or already done), 1 conflict, 2 usage, 3 refused, 4 busy (a held lock, or a ref that moved while `mr save` or `mr snapshot` ran: repeat later; a landing in flight: finish it first), 5 git or the disk failed (the report says what happened; follow `next:`). Add `--json` to any verb for one JSON object with `outcome`, `exit` and `next` (`--version --json` prints its own object, and `mortar --json guard-hook` prints nothing when it allows a move).

Three folder kinds exist. In a **request folder** all six verbs work. In a **zone folder** the agent verbs are refused. In an **attempt folder** (one of several tries at the same request) `status`, `save` and `undo` work, and `join`, `resolve` and `need` are refused.

## Installing for a user

Ask before you install anything. Then:

1. **Check the machine.** A Unix system (not Windows), `git --version` at 2.46 or newer, and a Rust toolchain (`cargo --version`).
2. **Build.**

       git clone https://github.com/codedaptive/mortar.git
       cd mortar
       cargo build --release --locked -p cli

3. **Install.** `./target/release/mortar install` copies the program to `~/.local/bin/mortar`, links `mr` to it, and offers to add a Mortar skill file to the agent harnesses it finds. Without a terminal it adds no skill unless you pass `--target <harness>` or `-y`. Ask the user which harnesses to wire before you pass either.
4. **Verify.** `mortar --version` prints one line. `mr help` lists the verbs.
5. **Say what changed:** the files install placed (it lists each one), and that `mortar uninstall` removes exactly those.

Safety rules:

- **Try Mortar in a scratch repository first.** The README's "A first run" makes one. Do not point a first run at the user's real repository.
- **`mr init` and `mr guard install` change a repository's git folder.** `mr init` writes a config there. `mr guard install` adds two git hooks that refuse direct moves of the origin and integration branches. Get the user's agreement before running either in a repository they care about.
- **Do not edit shell startup files without approval.** If `~/.local/bin` is not on the path, tell the user.
- **Do not delete lock files, the operation log or hooks by hand.** Mortar takes over a lock whose holder on this machine has died, and it never overwrites a hook it did not write.

## Troubleshooting

| What you see | What it means | What to do |
|---|---|---|
| `mr` runs another program, or is not found | Another tool owns `mr`, or the link was not made | Type `mortar` instead |
| `there is no Mortar config at ...` (exit 2) | `mr init` has not run in this repository | The harness or person who sets up the repository runs `mr init` |
| `this directory is not in a Mortar folder` (exit 3) | An agent verb was run outside a request folder | Run it inside the request folder |
| `this is a zone folder` (exit 3) | An agent verb was run in a zone's folder | Run it in a request folder |
| `the lock ... is held by pid ...` (exit 4) | Another Mortar command is running | Repeat the same command later |
| git says `only Mortar moves refs/heads/<branch>` | The guard refused a direct move of a protected branch | Bring work in with `mr join`, or ask for a landing |
| A document and the program disagree | The document is stale | Trust `mr help` and the command's own output, and say which you used |

## Completion checklist

Before you say Mortar is installed:

- `mortar --version` ran and printed a version.
- `mr help` (or `mortar help`) listed the verbs.
- You told the user which files were placed and which harnesses were wired, if any.
- You have not run `mr init` or `mr guard install` in a real repository without their agreement.
