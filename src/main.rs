use entropy_krypton_core::{
    EntropyMetrics,
    SentryConfig,
    SentryEngine,
    SentrySignals,
    SentryDecision,
};

fn main() {
    // Fake some samples like you'd get from telemetry:
    //   - positive / negative deltas, a bit noisy
    let samples = vec![0.05, -0.02, 0.04, 0.01, -0.03, 0.02];

    // Turn raw samples into entropy-style metrics.
    let metrics = EntropyMetrics::from_samples(&samples);

    // Map Krypton's metrics into the scores the sentry uses.
    let entropy_score = metrics.variance; // treat variance as "entropy-ish"
    let jitter_score = metrics.jitter;
    let load_score = 0.7; // fake load for the demo

    let signals = SentrySignals::from_raw(
    entropy_score,
    jitter_score,
    load_score,
    );

    // Use the default config for Krypton.
    let config = SentryConfig::default();
    let engine = SentryEngine::new(config);

    // Ask Krypton what to do with this job.
    let decision = engine.decide("job-42", &signals);

    println!("Krypton decision for job-42: {:?}", decision);

    match decision {
        SentryDecision::Keep => {
            println!("Action: let it run normally");
        }
        SentryDecision::Throttle => {
            println!("Action: slow it down / backoff");
        }
        SentryDecision::Kill => {
            println!("Action: kill or reschedule this job");
        }
    }
}
