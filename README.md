# krypton-consumer

`krypton-consumer` is a tiny CLI that drives the
[`krypton-entropy-core`](../krypton-entropy-core) library and streams
entropy stats + sentry decisions to stdout.

It samples the OS RNG (`OsRng`), maintains running metrics, and prints
per-iteration lines like:

```text
iter=000123 job=demo p=0.5312 mean=0.4978 var=0.003742 jitter=0.0324 n=124 decision=Keep
```

Where:

* `p` – bit density of the latest `u64` sample
* `mean` – running mean of `p`
* `variance` – running variance of `p`
* `jitter` – average absolute deviation
* `n` – total samples seen
* `decision` – `Keep`, `Throttle`, or `Kill` from Krypton

---

## Quickstart

From this crate:

```bash
cargo run -- --samples 1000 --job-id demo
```

Flags:

* `--samples N` – number of samples to take before exiting (default: 200)
* `--job-id ID` – optional job label that appears in the log lines

This binary is for demo / observability only. All entropy and decisions
come from `krypton-entropy-core` using `OsRng` as the sole entropy source.
