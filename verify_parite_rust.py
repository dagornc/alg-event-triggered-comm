#!/usr/bin/env python3
"""Compare bit-à-bit le binaire Rust et la référence Python."""

import argparse
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
REFERENCE = ROOT / "reference" / "sim_event_triggered_comm.py"
SCENARIOS = ("nominal", "turbulent", "alerts", "degraded")

def data_line(command: list[str]) -> str:
    output = subprocess.run(command, check=True, capture_output=True, text=True).stdout
    lines = output.strip().splitlines()
    if len(lines) != 2:
        raise RuntimeError(f"sortie CSV invalide ({len(lines)} lignes): {command}")
    return lines[1]

def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binaire", default=str(ROOT / "target" / "release" / "event_triggered_comm_rs"))
    parser.add_argument("--graines", nargs=2, type=int, default=[1000, 1050], metavar=("DEBUT", "FIN"))
    parser.add_argument("--scenarios", nargs="*", choices=SCENARIOS, default=list(SCENARIOS))
    args = parser.parse_args()
    binary = Path(args.binaire)
    if not binary.exists():
        print(f"ERREUR : binaire introuvable : {binary}", file=sys.stderr)
        return 2

    total = differences = 0
    common = ["30", "300", "6", "2", "3", "5"]
    for scenario in args.scenarios:
        identical = 0
        count = 0
        for seed in range(*args.graines):
            positional = [str(seed), *common, scenario]
            python_line = data_line([sys.executable, str(REFERENCE), *positional])
            rust_line = data_line([str(binary), *positional])
            total += 1
            count += 1
            if python_line == rust_line:
                identical += 1
            else:
                differences += 1
                if differences <= 10:
                    print(f"DIVERGENCE [{scenario}] seed={seed}")
                    print(f"  python: {python_line}")
                    print(f"  rust  : {rust_line}")
        print(f"scénario {scenario:10s} : {identical}/{count} identiques")

    print(f"TOTAL : {total - differences}/{total} identiques")
    print(f"COMPARAISONS : {total}")
    print(f"ÉCARTS : {differences}")
    if differences == 0:
        print("PARITÉ BIT-À-BIT VÉRIFIÉE ✓")
        return 0
    print("PARITÉ NON VÉRIFIÉE")
    return 1

if __name__ == "__main__":
    raise SystemExit(main())
