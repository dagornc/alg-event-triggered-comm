use event_triggered_comm_rs::{Params, Scenario, simuler};
use std::{env, process};

const HEADER: &str = "seed,scenario,agents,steps,threshold,hysteresis,guard_period,heartbeat_period,triggers,state_triggers,zone_crossings,alerts,periodic_messages,reduction_bp,avg_throughput_milli,avg_latency_us,max_latency_us";

fn usage(program: &str) -> ! {
    eprintln!(
        "usage: {program} SEED AGENTS STEPS THRESHOLD HYSTERESIS GUARD_PERIOD HEARTBEAT_PERIOD SCENARIO"
    );
    eprintln!("scenarios: nominal, turbulent, alerts, degraded");
    process::exit(2);
}

fn parse<T: std::str::FromStr>(value: &str, name: &str) -> T {
    value.parse().unwrap_or_else(|_| {
        eprintln!("invalid {name}: {value}");
        process::exit(2);
    })
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 9 {
        usage(&args[0]);
    }
    let seed = parse(&args[1], "seed");
    let agents = parse(&args[2], "agents");
    let steps = parse(&args[3], "steps");
    let threshold = parse(&args[4], "threshold");
    let hysteresis = parse(&args[5], "hysteresis");
    let guard_period = parse(&args[6], "guard_period");
    let heartbeat_period = parse(&args[7], "heartbeat_period");
    let scenario = Scenario::parse(&args[8]).unwrap_or_else(|| usage(&args[0]));
    if agents == 0
        || threshold <= 0
        || hysteresis < 0
        || hysteresis > threshold
        || heartbeat_period == 0
    {
        eprintln!(
            "agents, threshold and heartbeat_period must be positive; hysteresis must be in [0, threshold]"
        );
        process::exit(2);
    }
    let params = Params {
        agents,
        steps,
        threshold,
        hysteresis,
        guard_period,
        heartbeat_period,
        scenario,
    };
    let r = simuler(seed, &params);
    println!("{HEADER}");
    println!(
        "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
        seed,
        scenario.as_str(),
        agents,
        steps,
        threshold,
        hysteresis,
        guard_period,
        heartbeat_period,
        r.declenchements,
        r.declenchements_etat,
        r.franchissements_zone,
        r.alertes,
        r.messages_periodiques,
        r.reduction_bp,
        r.debit_moyen_milli,
        r.latence_moyenne_us,
        r.latence_max_us,
    );
}
