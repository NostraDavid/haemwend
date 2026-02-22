#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///

import argparse
import csv
import sys
from dataclasses import dataclass


@dataclass
class ScenarioMetric:
    name: str
    median: float
    minimum: float
    maximum: float


def read_metrics(path: str) -> dict[str, ScenarioMetric]:
    metrics: dict[str, ScenarioMetric] = {}
    with open(path, newline="", encoding="utf-8") as handle:
        reader = csv.DictReader(handle)
        for row in reader:
            name = row["name"]
            metrics[name] = ScenarioMetric(
                name=name,
                median=float(row["ns_per_step_median"]),
                minimum=float(row["ns_per_step_min"]),
                maximum=float(row["ns_per_step_max"]),
            )
    return metrics


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Vergelijk performance-resultaten van base en head."
    )
    parser.add_argument("--base", required=True, help="CSV van base branch")
    parser.add_argument("--head", required=True, help="CSV van head branch")
    parser.add_argument(
        "--max-regression-pct",
        type=float,
        default=10.0,
        help="Maximaal toegestane regressie in procent (default: 10)",
    )
    args = parser.parse_args()

    base = read_metrics(args.base)
    head = read_metrics(args.head)

    missing = sorted(set(base.keys()) ^ set(head.keys()))
    if missing:
        print("Scenario mismatch tussen base/head:")
        for name in missing:
            print(f"- {name}")
        return 2

    failing: list[str] = []

    print(
        f"Vergelijking (drempel: {args.max_regression_pct:.2f}%):\n"
        "scenario | base(ns/step) | head(ns/step) | delta(%) | status"
    )

    for name in sorted(base.keys()):
        base_metric = base[name]
        head_metric = head[name]

        if base_metric.median <= 0.0:
            print(f"{name} | ongeldig basismetriek")
            return 3

        delta_pct = ((head_metric.median - base_metric.median) / base_metric.median) * 100.0
        status = "OK"

        if delta_pct > args.max_regression_pct:
            status = "FAIL"
            failing.append(name)

        print(
            f"{name} | {base_metric.median:12.2f} | {head_metric.median:12.2f} | {delta_pct:8.2f}% | {status}"
        )

    if failing:
        print("\nPerformance regressie gedetecteerd in:")
        for name in failing:
            print(f"- {name}")
        return 1

    print("\nPerformance gate geslaagd.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
