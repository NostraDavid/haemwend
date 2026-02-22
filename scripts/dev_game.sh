#!/usr/bin/env bash
set -euo pipefail

watch_args=(
  -w src
  -w Cargo.toml
  -w Cargo.lock
)

if [ -d assets ]; then
  watch_args+=( -w assets )
fi

exec cargo watch "${watch_args[@]}" -x "run --features bevy/file_watcher --bin haemwend"
