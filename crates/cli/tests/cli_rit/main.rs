//! The cli_rit target holds no tests: every case it held ran the built mortar binary, and a run of the built
//! binary is validation, so they are in tests/validation. This folder still keeps the shared fixture and JSON
//! helpers (json.rs, support.rs and the fixture_*.rs files it includes) that validation, operational_rit and
//! resolve_rit include by path.
