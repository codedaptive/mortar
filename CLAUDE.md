# Mortar

Read the mission first: `~/dev_docs/forge-design/MISSION-mortar-2026-11-11.md`; the feature list
`~/dev_docs/forge-design/8-MORTAR-feature-list.md` is part of it.

- Build: `make check`. One crate's tests: `make zone-test-one DIR=crates/<crate>` (GROUP=unit by default).
- Code: `tessera/docs/CHARTER-small-code-files-2026-10-30.md` (one public symbol per file, ~150 lines).
- Every test makes its own throwaway repository and removes it. Never run against a real team's repository.
- Rulings: `docs/RULINGS.md` (only Bob edits it).
