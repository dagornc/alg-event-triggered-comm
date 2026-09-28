//! Simulateur discret d'une politique d'émission déclenchée par événement.
//!
//! Chaque agent maintient l'état connu de ses destinataires. Une émission est
//! demandée lorsqu'une excursion dépasse le seuil, lors d'un changement de
//! zone ou pour une alerte critique. La garde s'applique aux événements
//! ordinaires ; une alerte critique la contourne.

use crate::rng::PyRandom;

pub const DEFAULT_AGENTS: usize = 30;
const ZONE_WIDTH: i64 = 15;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Scenario {
    Nominal,
    Turbulent,
    Alerts,
    Degraded,
}

impl Scenario {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Nominal => "nominal",
            Self::Turbulent => "turbulent",
            Self::Alerts => "alerts",
            Self::Degraded => "degraded",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "nominal" => Some(Self::Nominal),
            "turbulent" => Some(Self::Turbulent),
            "alerts" => Some(Self::Alerts),
            "degraded" => Some(Self::Degraded),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Params {
    pub agents: usize,
    pub steps: usize,
    pub threshold: i64,
    pub hysteresis: i64,
    pub guard_period: usize,
    pub heartbeat_period: usize,
    pub scenario: Scenario,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            agents: DEFAULT_AGENTS,
            steps: 300,
            threshold: 6,
            hysteresis: 2,
            guard_period: 3,
            heartbeat_period: 5,
            scenario: Scenario::Nominal,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Resultat {
    pub declenchements: u64,
    pub declenchements_etat: u64,
    pub franchissements_zone: u64,
    pub alertes: u64,
    pub messages_periodiques: u64,
    pub reduction_bp: i64,
    pub debit_moyen_milli: u64,
    pub latence_moyenne_us: u64,
    pub latence_max_us: u64,
}

#[derive(Clone, Copy)]
struct Agent {
    state: i64,
    last_sent: i64,
    zone: i64,
    state_event_active: bool,
    last_tx: Option<usize>,
}

impl Default for Agent {
    fn default() -> Self {
        Self {
            state: 0,
            last_sent: 0,
            zone: zone_of(0),
            state_event_active: false,
            last_tx: None,
        }
    }
}

fn zone_of(state: i64) -> i64 {
    state.div_euclid(ZONE_WIDTH)
}

fn guard_elapsed(last_tx: Option<usize>, step: usize, guard_period: usize) -> bool {
    last_tx.is_none_or(|last| step.saturating_sub(last) >= guard_period)
}

/// Simule la politique pour une graine et des paramètres donnés.
pub fn simuler(seed: u64, params: &Params) -> Resultat {
    let mut rng = PyRandom::new(seed);
    let mut agents = vec![Agent::default(); params.agents];
    let threshold = params.threshold.max(1);
    let low_threshold = (threshold - params.hysteresis.clamp(0, threshold)).max(0);
    let mut declenchements_etat = 0u64;
    let mut franchissements_zone = 0u64;
    let mut alertes = 0u64;
    let mut total_latency = 0u64;
    let mut latency_max = 0u64;

    for step in 0..params.steps {
        let mut emissions_this_step: Vec<(usize, Trigger)> = Vec::new();
        for (index, agent) in agents.iter_mut().enumerate() {
            let amplitude = if params.scenario == Scenario::Turbulent {
                3
            } else {
                1
            };
            agent.state += rng.randint(-amplitude, amplitude);

            let error = (agent.state - agent.last_sent).abs();
            if agent.state_event_active && error <= low_threshold {
                agent.state_event_active = false;
            }
            let state_event = !agent.state_event_active && error >= threshold;
            if state_event {
                agent.state_event_active = true;
            }

            let current_zone = zone_of(agent.state);
            let zone_event = current_zone != agent.zone;
            let alert = params.scenario == Scenario::Alerts && rng.randint(0, 99) < 4;
            let ordinary_allowed = guard_elapsed(agent.last_tx, step, params.guard_period);
            let trigger = if alert {
                Some(Trigger::Alert)
            } else if zone_event && ordinary_allowed {
                Some(Trigger::Zone)
            } else if state_event && ordinary_allowed {
                Some(Trigger::State)
            } else {
                None
            };

            if let Some(kind) = trigger {
                agent.last_sent = agent.state;
                agent.zone = current_zone;
                agent.last_tx = Some(step);
                emissions_this_step.push((index, kind));
            }
        }

        let congestion = emissions_this_step.len() as u64 * 250;
        let base_latency = if params.scenario == Scenario::Degraded {
            30_000
        } else {
            10_000
        };
        let latency = base_latency + congestion;
        for (_, kind) in emissions_this_step {
            match kind {
                Trigger::State => declenchements_etat += 1,
                Trigger::Zone => franchissements_zone += 1,
                Trigger::Alert => alertes += 1,
            }
            total_latency += latency;
            latency_max = latency_max.max(latency);
        }
    }

    let declenchements = declenchements_etat + franchissements_zone + alertes;
    let period = params.heartbeat_period.max(1);
    let beats = params.steps.div_ceil(period) as u64;
    let messages_periodiques = params.agents as u64 * beats;
    let reduction_bp = if messages_periodiques == 0 {
        0
    } else {
        (messages_periodiques as i64 - declenchements as i64) * 10_000 / messages_periodiques as i64
    };
    Resultat {
        declenchements,
        declenchements_etat,
        franchissements_zone,
        alertes,
        messages_periodiques,
        reduction_bp,
        debit_moyen_milli: if params.steps == 0 {
            0
        } else {
            declenchements * 1_000 / params.steps as u64
        },
        latence_moyenne_us: if declenchements == 0 {
            0
        } else {
            total_latency / declenchements
        },
        latence_max_us: latency_max,
    }
}

#[derive(Clone, Copy)]
enum Trigger {
    State,
    Zone,
    Alert,
}
