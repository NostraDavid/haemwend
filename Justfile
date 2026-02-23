set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

default:
  @just --list

run:
  cargo run --bin haemwend

run-release:
  cargo run --release --bin haemwend

run-hot:
  ./scripts/dev_game.sh

blender-ui:
  cargo run --bin blender_model_editor_ui

blender-ui-hot:
  cargo watch -w src -w config -w Cargo.toml -w Cargo.lock -x "run --features bevy/file_watcher --bin blender_model_editor_ui"

blender-table-validate:
  blender -b --factory-startup --python assets/blender_ai/table.py -- --validate

blender-table-artifacts out_dir="assets/blender_ai/_artifacts":
  ts=$(date +"%Y%m%d_%H%M%S"); \
  render="{{out_dir}}/table_render_${ts}.png"; \
  report="{{out_dir}}/table_report_${ts}.json"; \
  mkdir -p "{{out_dir}}"; \
  blender -b --factory-startup --python assets/blender_ai/table.py -- --validate --render "${render}" --report-json "${report}"; \
  echo "render: ${render}"; \
  echo "report: ${report}"

blender-table-tune opts="" out_dir="assets/blender_ai/_artifacts":
  ts=$(date +"%Y%m%d_%H%M%S"); \
  render="{{out_dir}}/table_render_${ts}.png"; \
  report="{{out_dir}}/table_report_${ts}.json"; \
  mkdir -p "{{out_dir}}"; \
  blender -b --factory-startup --python assets/blender_ai/table.py -- \
    --validate \
    {{opts}} \
    --render "${render}" \
    --report-json "${report}"; \
  echo "render: ${render}"; \
  echo "report: ${report}"

# generic export:
#   just blender-export <script.py> [out.glb]
# examples:
#   just blender-export
#   just blender-export assets/blender_ai/table.py assets/models/table_v2.glb
blender-export script="assets/blender_ai/table.py" out="" opts="":
  name=$(basename "{{script}}" .py); \
  out_path="{{out}}"; \
  if [ -z "${out_path}" ]; then out_path="assets/models/${name}.glb"; fi; \
  blender -b --factory-startup --python "{{script}}" -- --validate {{opts}} --export-glb "${out_path}"; \
  echo "glb: ${out_path}"

blender-table-export out="" opts="":
  name=$(basename "assets/blender_ai/table.py" .py); \
  out_path="{{out}}"; \
  if [ -z "${out_path}" ]; then out_path="assets/models/${name}.glb"; fi; \
  blender -b --factory-startup --python "assets/blender_ai/table.py" -- --validate {{opts}} --export-glb "${out_path}"; \
  echo "glb: ${out_path}"

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

# clean local git object/lfs cache to shrink .git size
git-clean:
  git gc --prune=now
  git lfs prune

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
