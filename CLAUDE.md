# Mortar

What Mortar is and how to work on it: `README.md` and `CONTRIBUTING.md`.

- Build: `make check`. One crate's tests: `make test-one DIR=crates/<crate>` (GROUP=unit by default).
- Code: one public symbol per file, named after it, about 150 lines at most.
- Every test makes its own throwaway repository and removes it. Never run against a real team's repository.
- Rulings: `docs/RULINGS.md` (the maintainer's decisions; edited by the maintainer only).
