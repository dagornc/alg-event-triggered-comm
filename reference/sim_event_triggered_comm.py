#!/usr/bin/env python3
"""Référence normative miroir pour ALG_EVENT_TRIGGERED_COMM."""

import random
import sys
from dataclasses import dataclass

HEADER = "seed,scenario,agents,steps,threshold,hysteresis,guard_period,heartbeat_period,triggers,state_triggers,zone_crossings,alerts,periodic_messages,reduction_bp,avg_throughput_milli,avg_latency_us,max_latency_us"
SCENARIOS = {"nominal", "turbulent", "alerts", "degraded"}
ZONE_WIDTH = 15

@dataclass
class Agent:
    state: int = 0
    last_sent: int = 0
    zone: int = 0
    state_event_active: bool = False
    last_tx: int | None = None

def simulate(seed: int, agents_count: int, steps: int, threshold: int, hysteresis: int,
             guard_period: int, heartbeat_period: int, scenario: str) -> tuple[int, ...]:
    rng = random.Random(seed)
    agents = [Agent() for _ in range(agents_count)]
    low_threshold = max(threshold - min(max(hysteresis, 0), threshold), 0)
    state_triggers = zone_crossings = alerts = 0
    total_latency = max_latency = 0

    for step in range(steps):
        emissions: list[str] = []
        for agent in agents:
            amplitude = 3 if scenario == "turbulent" else 1
            agent.state += rng.randint(-amplitude, amplitude)
            error = abs(agent.state - agent.last_sent)
            if agent.state_event_active and error <= low_threshold:
                agent.state_event_active = False
            state_event = not agent.state_event_active and error >= threshold
            if state_event:
                agent.state_event_active = True
            current_zone = agent.state // ZONE_WIDTH
            zone_event = current_zone != agent.zone
            alert = scenario == "alerts" and rng.randint(0, 99) < 4
            ordinary_allowed = agent.last_tx is None or step - agent.last_tx >= guard_period
            trigger = None
            if alert:
                trigger = "alert"
            elif zone_event and ordinary_allowed:
                trigger = "zone"
            elif state_event and ordinary_allowed:
                trigger = "state"
            if trigger is not None:
                agent.last_sent = agent.state
                agent.zone = current_zone
                agent.last_tx = step
                emissions.append(trigger)

        latency = (30_000 if scenario == "degraded" else 10_000) + len(emissions) * 250
        for trigger in emissions:
            if trigger == "state":
                state_triggers += 1
            elif trigger == "zone":
                zone_crossings += 1
            else:
                alerts += 1
            total_latency += latency
            max_latency = max(max_latency, latency)

    triggers = state_triggers + zone_crossings + alerts
    beats = (steps + heartbeat_period - 1) // heartbeat_period
    periodic_messages = agents_count * beats
    reduction_bp = 0 if periodic_messages == 0 else (periodic_messages - triggers) * 10_000 // periodic_messages
    throughput = 0 if steps == 0 else triggers * 1_000 // steps
    avg_latency = 0 if triggers == 0 else total_latency // triggers
    return (triggers, state_triggers, zone_crossings, alerts, periodic_messages,
            reduction_bp, throughput, avg_latency, max_latency)

def main(argv: list[str]) -> int:
    if len(argv) != 9:
        print(f"usage: {argv[0]} SEED AGENTS STEPS THRESHOLD HYSTERESIS GUARD_PERIOD HEARTBEAT_PERIOD SCENARIO", file=sys.stderr)
        return 2
    try:
        seed, agents, steps, threshold, hysteresis, guard, heartbeat = map(int, argv[1:8])
    except ValueError as exc:
        print(f"invalid integer: {exc}", file=sys.stderr)
        return 2
    scenario = argv[8]
    if scenario not in SCENARIOS or agents <= 0 or threshold <= 0 or not 0 <= hysteresis <= threshold or heartbeat <= 0:
        print("invalid parameters", file=sys.stderr)
        return 2
    result = simulate(seed, agents, steps, threshold, hysteresis, guard, heartbeat, scenario)
    print(HEADER)
    prefix = (seed, scenario, agents, steps, threshold, hysteresis, guard, heartbeat)
    print(",".join(map(str, prefix + result)))
    return 0

if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
