set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

default:
  @just --list

run:
  cargo run --bin haemwend

run-hot:
  ./scripts/dev_game.sh

check:
  cargo check

check-hot:
  cargo check --features bevy/file_watcher

fmt:
  cargo fmt --all

fmt-check:
  cargo fmt --all -- --check

clippy:
  cargo clippy --all-targets --all-features -- -D warnings

test:
  cargo test

ci:
  cargo check
  cargo fmt --all -- --check
  cargo clippy --all-targets --all-features -- -D warnings
  cargo test

perf output="perf_results.csv" repeats="3" warmup="1":
  cargo run --release --bin perf_scenarios -- --scenario-file perf/scenarios.ron --output {{output}} --warmup {{warmup}} --repeats {{repeats}}

compare base head threshold="10":
  uv run scripts/compare_perf_results.py --base {{base}} --head {{head}} --max-regression-pct {{threshold}}
