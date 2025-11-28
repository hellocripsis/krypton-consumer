# krypton-consumer

Tiny Rust binary that demonstrates how to use
[`entropy-krypton-core`](https://github.com/hellocripsis/entropy-sentry-core)
as a job policy engine.

It:

- Depends on `entropy-krypton-core` via a local path.
- Builds a `SentryEngine` with a simple config.
- Feeds in fake telemetry for a single job (`job-42`).
- Prints the decision: Keep / Throttle / Kill, plus the action.

Run:

```bash
cargo run
