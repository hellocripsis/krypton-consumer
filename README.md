# krypton-consumer

Small Rust binary that shows how to use
[`entropy-krypton-core`](https://github.com/hellocripsis/entropy-sentry-core)
as a job policy engine.

What it does:

- Depends on `entropy-krypton-core` as a local library.
- Builds a `SentryEngine` with simple thresholds.
- Feeds in fake telemetry for a single job (`job-42`).
- Prints a decision: `Keep`, `Throttle`, or `Kill`, plus the suggested action.

Run it:

```bash
cargo run
