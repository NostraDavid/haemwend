#!/usr/bin/env bash
set -euo pipefail

watch_args=(
  -w src
  -w Cargo.toml
  -w Cargo.lock
)

exec cargo watch "${watch_args[@]}" -x "run --features bevy/file_watcher --bin haemwend"
