# Changelog

All notable changes to this project are documented in this file.

## [0.1.0] - 2026-03-05

- Added a real CLI that streams entropy/sentry observations from `OsRng`.
- Added `--samples`, `--job-id`, `--load-score`, `--config`, and `--format` flags.
- Added JSON output mode with logical field ordering for machine-readable ingestion.
- Switched `krypton-entropy-core` to a pinned Git dependency (`v0.1.0`) for reproducible builds.
- Added CI workflow with caching, `--locked` builds, format, lint, test, and build checks.
- Added `description`, `license`, `repository`, and `rust-version` to `Cargo.toml`.
- Added `LICENSE` (MIT) and this `CHANGELOG`.
- Hardened argument parsing: duplicate flags, empty `--job-id`, and out-of-range `--load-score` are all rejected with clear error messages.
