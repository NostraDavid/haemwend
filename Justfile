set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

default:
  @just --list

run:
  cargo run --bin haemwend

run-release:
  cargo run --release --bin haemwend

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

# optimize pngs without losing quality; better than pngquant, but may not be as good as pngquant in terms of file size reduction
oxipng-optimize:
  find . -type f -iname '*.png' -not -path './.git/*' -print0 | \
    xargs -0 -r oxipng -o max --strip safe --alpha --zopfli

# optimize pngs by losing quality (reducing the palette to 256 colors); better than oxipng, but may cause some visual artifacts
pngquant-optimize:
  status=0; \
  while IFS= read -r -d '' file; do \
    if pngquant --force --ext .png --skip-if-larger --speed 1 --quality 0-100 --strip -- "$file"; then \
      :; \
    else \
      code=$?; \
      if [ "$code" -ne 98 ] && [ "$code" -ne 99 ]; then \
        echo "pngquant faalde voor $file (exit $code)" >&2; \
        status=1; \
      fi; \
    fi; \
  done < <(find . -type f -iname '*.png' -not -path './.git/*' -print0); \
  exit $status
