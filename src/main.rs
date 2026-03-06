use krypton_entropy_core::{SentryConfig, SentryDecision, SentryEngine, SentrySignals};
use rand::{rngs::OsRng, RngCore};
use serde::Serialize;
use std::{env, path::PathBuf, process};

fn main() {
    if let Err(msg) = run() {
        eprintln!("{msg}");
        process::exit(2);
    }
}

fn run() -> Result<(), String> {
    let args = parse_args(env::args().skip(1))?;
    let config = load_config(args.config_path.as_ref())?;
    let engine = SentryEngine::new(config);

    let mut rng = OsRng;
    let mut stats = RunningStats::default();
    let load_score = args.load_score;

    for iter in 0..args.samples {
        let sample = rng.next_u64();
        let p = f64::from(sample.count_ones()) / 64.0;

        stats.push(p);
        let var = stats.variance();
        let jitter = stats.jitter();

        let signals = SentrySignals::from_raw(var, jitter, load_score);
        let decision = engine.decide(&args.job_id, &signals);
        let decision_name = decision_label(&decision);

        match args.output_format {
            OutputFormat::Text => {
                println!(
                    "iter={:06} job={} p={:.4} mean={:.4} var={:.6} jitter={:.4} n={} decision={}",
                    iter, args.job_id, p, stats.mean, var, jitter, stats.n, decision_name,
                );
            }
            OutputFormat::Json => {
                let record = Record {
                    iter,
                    job: &args.job_id,
                    p,
                    mean: stats.mean,
                    var,
                    jitter,
                    n: stats.n,
                    decision: decision_name,
                };
                println!(
                    "{}",
                    serde_json::to_string(&record).expect("serialization is infallible")
                );
            }
        }
    }

    Ok(())
}

/// One output row — field declaration order is preserved by serde.
#[derive(Serialize)]
struct Record<'a> {
    iter: usize,
    job: &'a str,
    p: f64,
    mean: f64,
    var: f64,
    jitter: f64,
    n: u64,
    decision: &'a str,
}

#[derive(Debug, Clone)]
struct Args {
    samples: usize,
    job_id: String,
    config_path: Option<PathBuf>,
    load_score: f64,
    output_format: OutputFormat,
}

#[derive(Debug, Clone, Copy)]
enum OutputFormat {
    Text,
    Json,
}

impl OutputFormat {
    fn parse(s: &str) -> Result<Self, String> {
        match s {
            "text" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            other => Err(format!(
                "Invalid --format value: {other}. Expected one of: text, json"
            )),
        }
    }
}

fn usage() -> String {
    let bin = env::args()
        .next()
        .unwrap_or_else(|| "krypton-consumer".to_string());
    format!(
        "Usage:\n  {bin} [--samples N] [--job-id ID] [--load-score F] [--config PATH] [--format text|json]\n\nFlags:\n  --samples N      number of samples to take before exiting (default: 200)\n  --job-id ID      job label that appears in the log lines (default: demo)\n  --load-score F   static load score in [0.0, 1.0] used in sentry signals (default: 0.7)\n  --config PATH    optional JSON config path for SentryConfig\n  --format MODE    output format: text or json (default: text)\n  -h, --help       show this help text"
    )
}

fn parse_args<I>(mut it: I) -> Result<Args, String>
where
    I: Iterator<Item = String>,
{
    let mut args = Args {
        samples: 200,
        job_id: "demo".to_string(),
        config_path: None,
        load_score: 0.7_f64,
        output_format: OutputFormat::Text,
    };

    // Track which flags have already been set to catch duplicates.
    let mut seen_samples = false;
    let mut seen_job_id = false;
    let mut seen_load_score = false;
    let mut seen_config = false;
    let mut seen_format = false;

    while let Some(tok) = it.next() {
        match tok.as_str() {
            "-h" | "--help" => {
                println!("{}", usage());
                process::exit(0);
            }
            "--samples" => {
                if seen_samples {
                    return Err("--samples specified more than once".to_string());
                }
                seen_samples = true;
                let raw = it
                    .next()
                    .ok_or_else(|| "Missing value for --samples".to_string())?;
                args.samples = raw
                    .parse::<usize>()
                    .map_err(|_| format!("Invalid --samples value: {raw}"))?;
                if args.samples == 0 {
                    return Err("--samples must be >= 1".to_string());
                }
            }
            "--job-id" => {
                if seen_job_id {
                    return Err("--job-id specified more than once".to_string());
                }
                seen_job_id = true;
                let raw = it
                    .next()
                    .ok_or_else(|| "Missing value for --job-id".to_string())?;
                if raw.is_empty() {
                    return Err("--job-id must not be empty".to_string());
                }
                args.job_id = raw;
            }
            "--load-score" => {
                if seen_load_score {
                    return Err("--load-score specified more than once".to_string());
                }
                seen_load_score = true;
                let raw = it
                    .next()
                    .ok_or_else(|| "Missing value for --load-score".to_string())?;
                let val = raw
                    .parse::<f64>()
                    .map_err(|_| format!("Invalid --load-score value: {raw}"))?;
                if !val.is_finite() || !(0.0..=1.0).contains(&val) {
                    return Err(format!(
                        "--load-score must be a finite number in [0.0, 1.0], got {raw}"
                    ));
                }
                args.load_score = val;
            }
            "--config" => {
                if seen_config {
                    return Err("--config specified more than once".to_string());
                }
                seen_config = true;
                let raw = it
                    .next()
                    .ok_or_else(|| "Missing value for --config".to_string())?;
                args.config_path = Some(PathBuf::from(raw));
            }
            "--format" => {
                if seen_format {
                    return Err("--format specified more than once".to_string());
                }
                seen_format = true;
                let raw = it
                    .next()
                    .ok_or_else(|| "Missing value for --format".to_string())?;
                args.output_format = OutputFormat::parse(&raw)?;
            }
            other => return Err(format!("Unknown argument: {other}\n\n{}", usage())),
        }
    }

    Ok(args)
}

fn load_config(path: Option<&PathBuf>) -> Result<SentryConfig, String> {
    match path {
        Some(path) => SentryConfig::from_json_file(path)
            .map_err(|e| format!("Failed to load config from {}: {e}", path.display())),
        None => Ok(SentryConfig::default()),
    }
}

#[derive(Debug, Default, Clone)]
struct RunningStats {
    n: u64,
    mean: f64,
    m2: f64,
    abs_dev_sum: f64,
}

impl RunningStats {
    fn push(&mut self, x: f64) {
        self.n += 1;
        let n = self.n as f64;

        let delta = x - self.mean;
        self.mean += delta / n;
        let delta2 = x - self.mean;
        self.m2 += delta * delta2;

        // Deviation is computed from the updated running mean; this slightly
        // underestimates true MAD but is acceptable for this demo context.
        self.abs_dev_sum += (x - self.mean).abs();
    }

    fn variance(&self) -> f64 {
        if self.n == 0 {
            0.0
        } else {
            self.m2 / (self.n as f64)
        }
    }

    fn jitter(&self) -> f64 {
        if self.n == 0 {
            0.0
        } else {
            self.abs_dev_sum / (self.n as f64)
        }
    }
}

fn decision_label(d: &SentryDecision) -> &'static str {
    match d {
        SentryDecision::Keep => "Keep",
        SentryDecision::Throttle => "Throttle",
        SentryDecision::Kill => "Kill",
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_args, OutputFormat};

    fn parse(raw: &[&str]) -> Result<super::Args, String> {
        parse_args(raw.iter().map(|s| s.to_string()))
    }

    // --- happy path ---

    #[test]
    fn parses_defaults() {
        let args = parse(&[]).expect("parse should succeed");
        assert_eq!(args.samples, 200);
        assert_eq!(args.job_id, "demo");
        assert_eq!(args.load_score, 0.7_f64);
        assert!(matches!(args.output_format, OutputFormat::Text));
        assert!(args.config_path.is_none());
    }

    #[test]
    fn parses_custom_values() {
        let args = parse(&[
            "--samples",
            "12",
            "--job-id",
            "batch-7",
            "--load-score",
            "0.42",
            "--format",
            "json",
            "--config",
            "/tmp/krypton.json",
        ])
        .expect("parse should succeed");

        assert_eq!(args.samples, 12);
        assert_eq!(args.job_id, "batch-7");
        assert_eq!(args.load_score, 0.42);
        assert!(matches!(args.output_format, OutputFormat::Json));
        assert_eq!(
            args.config_path.as_deref().and_then(|p| p.to_str()),
            Some("/tmp/krypton.json")
        );
    }

    #[test]
    fn parses_boundary_samples_one() {
        let args = parse(&["--samples", "1"]).expect("samples=1 should succeed");
        assert_eq!(args.samples, 1);
    }

    #[test]
    fn parses_boundary_load_score_zero() {
        let args = parse(&["--load-score", "0.0"]).expect("load-score=0.0 should succeed");
        assert_eq!(args.load_score, 0.0);
    }

    #[test]
    fn parses_boundary_load_score_one() {
        let args = parse(&["--load-score", "1.0"]).expect("load-score=1.0 should succeed");
        assert_eq!(args.load_score, 1.0);
    }

    // --- error paths ---

    #[test]
    fn rejects_unknown_format() {
        let err = parse(&["--format", "yaml"]).expect_err("parse should fail");
        assert!(err.contains("Invalid --format value"));
    }

    #[test]
    fn rejects_samples_zero() {
        let err = parse(&["--samples", "0"]).expect_err("samples=0 should fail");
        assert!(err.contains("must be >= 1"));
    }

    #[test]
    fn rejects_empty_job_id() {
        let err = parse(&["--job-id", ""]).expect_err("empty job-id should fail");
        assert!(err.contains("must not be empty"));
    }

    #[test]
    fn rejects_load_score_negative() {
        let err = parse(&["--load-score", "-0.1"]).expect_err("negative load-score should fail");
        assert!(err.contains("[0.0, 1.0]"));
    }

    #[test]
    fn rejects_load_score_above_one() {
        let err = parse(&["--load-score", "1.1"]).expect_err("load-score > 1 should fail");
        assert!(err.contains("[0.0, 1.0]"));
    }

    #[test]
    fn rejects_load_score_nan() {
        let err = parse(&["--load-score", "NaN"]).expect_err("NaN load-score should fail");
        assert!(err.contains("[0.0, 1.0]"));
    }

    #[test]
    fn rejects_duplicate_samples() {
        let err = parse(&["--samples", "2", "--samples", "5"]).expect_err("duplicate should fail");
        assert!(err.contains("more than once"));
    }

    #[test]
    fn rejects_duplicate_job_id() {
        let err = parse(&["--job-id", "a", "--job-id", "b"]).expect_err("duplicate should fail");
        assert!(err.contains("more than once"));
    }

    #[test]
    fn rejects_duplicate_load_score() {
        let err = parse(&["--load-score", "0.5", "--load-score", "0.6"])
            .expect_err("duplicate should fail");
        assert!(err.contains("more than once"));
    }

    #[test]
    fn rejects_duplicate_format() {
        let err =
            parse(&["--format", "text", "--format", "json"]).expect_err("duplicate should fail");
        assert!(err.contains("more than once"));
    }

    #[test]
    fn rejects_unknown_flag() {
        let err = parse(&["--unknown"]).expect_err("unknown flag should fail");
        assert!(err.contains("Unknown argument"));
    }

    #[test]
    fn rejects_missing_samples_value() {
        let err = parse(&["--samples"]).expect_err("missing value should fail");
        assert!(err.contains("Missing value for --samples"));
    }

    // --- JSON field ordering ---

    #[test]
    fn json_record_fields_are_in_logical_order() {
        let record = super::Record {
            iter: 0,
            job: "test",
            p: 0.5,
            mean: 0.5,
            var: 0.0,
            jitter: 0.0,
            n: 1,
            decision: "Keep",
        };
        let s = serde_json::to_string(&record).unwrap();
        // With the preserve_order feature, serde_json::Map retains insertion order.
        let obj: serde_json::Map<String, serde_json::Value> = serde_json::from_str(&s).unwrap();
        let keys: Vec<&str> = obj.keys().map(String::as_str).collect();
        assert_eq!(
            keys,
            ["iter", "job", "p", "mean", "var", "jitter", "n", "decision"]
        );
    }
}
