# Changelog

All notable changes to this project are documented in this file.

## [0.1.0] - 2026-03-05

- Added a real CLI that streams entropy/sentry observations from `OsRng`.
- Added `--samples`, `--job-id`, `--load-score`, `--config`, and `--format` flags.
- Added JSON output mode for machine-readable ingestion.
- Switched `krypton-entropy-core` to a pinned Git dependency (`v0.1.0`) for reproducible builds.
- Added CI workflow for format, lint, test, and build checks.
