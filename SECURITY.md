# Security Policy

Mortar moves the branches that hold a team's proven code, and it runs inside the repositories of the agents that write that code. If you find a problem that touches that trust, please report it privately.

## What Mortar does to stay safe

These are properties of the code, not of a layer added around it.

- **It runs git as a program, with no shell.** Every git call is a program and a list of arguments. Mortar links no git library and has no runtime dependency outside the Rust standard library.
- **It opens no network connection of its own.** The only network use is git's fetch, push and ls-remote against the one remote the repository's config names. There is no telemetry.
- **Every move of a branch, a snapshot ref or a squash ref names the value it expects to replace.** A move whose ref is not where Mortar read it is refused and nothing moves, so two movers cannot overwrite each other.
- **The origin branch only moves forward.** Mortar refuses any move of it that is not a fast-forward, and its push to the remote is leased at the value it read.
- **Every such move is written down first.** Mortar appends its intent to the operation log before it moves the ref and a closing record after. The copies a fetch leaves under `refs/mortar/fetched/` and `refs/mortar-backup/` are the exception: `git fetch` writes them with no record. A run that was killed in between is settled from the refs by the next command that takes a lock, before it does anything else.
- **Names are checked before they reach git.** A zone or request name is letters, digits, `-`, `_` and `.` only, so it cannot leave its place in a branch name or be read as an option. Every ref name is checked before any write.
- **The guard fails closed.** When its hooks cannot read their input or their config, the move is refused. The environment variable Mortar sets on its own moves allows nothing by itself: the move must also be listed in an open record in the operation log.
- **It never overwrites a git hook it did not write.** `mr guard install` refuses and names the file.
- **A restore from backup cannot reach beyond in-flight work.** Only zone and request branches, snapshots and squashes are backed up or restored, so a stray backup ref cannot become the main branch.

## What this does not defend against

Limits, as plainly as possible.

**The guard stops accidents, not a determined bypass.** It is two git hooks in the repository. Anyone who can write to the repository's git folder can remove the hooks, edit the config or the operation log, or move a ref by writing its file. The guard exists so that an agent or a script that types `git push` or `git update-ref` by mistake is refused and told what to do instead. It compares a push URL with the configured remote's URLs as plain text, so the same remote reached by a different spelling of its URL is not recognised. Protect the branch on the server as well if you need more than that.

**Your machine is the boundary.** Mortar's config, operation log, locks and folder records are ordinary files owned by your user account. Any process running as you can read and change them. Mortar adds no access control between agents on one machine: an agent that can run `mr` in a folder can run every verb there, the landing verbs included. Which agent may run which verb is the harness's to enforce.

**The config names programs that Mortar runs.** `mr init --formatter` stores a command in the repository's git folder, and every later save, join and snapshot runs it over the changed files, as the user who typed the command and with that user's environment. `mr bisect` runs the test command it is handed, in the same way. Treat write access to `<git dir>/mortar/config.json` as the right to run a program as whoever next saves.

**The guard's hooks find `mortar` on the path.** The hooks are one line: they run `mortar guard-hook`. Whatever program is first on the path under that name is what git runs. Keep the folder that holds `mortar` writable only by you.

**Mortar holds no opinion about the code it moves.** It does not scan, test or review. A commit that lands through Mortar is exactly as trustworthy as the checks the caller ran before asking for the landing.

**The operation log is a record, not a proof.** It is a plain file. It tells an honest operator what happened and lets Mortar recover from a kill. It is not signed and does not detect tampering.

## Verifying what you run

Mortar is built from source; there are no signed release binaries. Build from a commit you have checked out and inspected:

```
cargo build --release --locked -p cli
```

`--locked` builds with exactly the dependency versions in `Cargo.lock`. `mortar --version` prints the version and the commit the program was built from.

`mortar install` reads the files it finds under the names `mortar` and `mr`; it never runs them. `mortar uninstall` removes only the files install recorded, and only while each is still what install placed.

## Reporting a vulnerability

**Do not open a public issue for a security problem.**

Use GitHub's [Report a vulnerability](https://github.com/codedaptive/mortar/security/advisories/new) form under this repository's Security tab, or email <security@codedaptive.com>.

A useful report names the verb or crate affected, the version or commit, and enough detail to reproduce what you observed.

## What counts

Anything that breaks a property listed under "What Mortar does to stay safe": a protected branch moved backward or sideways through Mortar, a ref overwritten without its expected value, a move the guard should have refused, a name or path that reaches git as an option or escapes its place, a command run that the config did not name, or work lost by a verb that reported success.

A way around the guard that needs write access to the repository's git folder is a known limit, described above, not a vulnerability.

## What to expect

Mortar is maintained on a best-effort basis. We read security reports, work confirmed problems privately, and disclose once a fix has shipped. We are glad to credit reporters. There is no bug bounty and no response-time guarantee.

## Supported versions

| Version | Supported |
|---|---|
| 0.1.x | Yes: the current line |
