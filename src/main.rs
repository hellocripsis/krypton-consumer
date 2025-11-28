use entropy_krypton_core::{SentryConfig, SentryDecision, SentryEngine, SentrySignals};

fn main() {
    // 1) Configure Krypton: how strict we are
    let config = SentryConfig {
        max_entropy_score: 2.0,
        soft_excess_factor: 1.0,
        hard_excess_factor: 1.5,
    };
    let engine = SentryEngine::new(config);

    // 2) Pretend these came from telemetry (latency/jitter/load)
    let entropy_score = 0.6;
    let jitter_score = 0.4;
    let load_score = 0.7;

    let signals = SentrySignals::from_raw(entropy_score, jitter_score, load_score);

    // 3) Ask Krypton what to do
    let decision = engine.decide("job-42", &signals);

    println!("Krypton decision for job-42: {:?}", decision);

    match decision {
        SentryDecision::Keep => {
            println!("Action: let it run normally");
        }
        SentryDecision::Throttle => {
            println!("Action: slow it down / reduce concurrency");
        }
        SentryDecision::Kill => {
            println!("Action: terminate or quarantine this job");
        }
    }
}
