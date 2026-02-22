# Performance gate in GitHub Actions

Dit project gebruikt een CI-gate voor performance regressies op pull requests.

## Wat deze workflow doet
- Draait deterministische benchmark-scenario's op de `head` commit van de PR.
- Draait dezelfde scenario's op de `base` commit van de PR.
- Vergelijkt `ns_per_step_median` per scenario.
- Faalt de workflow als een scenario meer dan `10%` trager is.
- Als `base` de benchmark-harness nog niet heeft (eerste introductie-PR), wordt vergelijken automatisch overgeslagen.
- Draait het vergelijkscript via `uv run`.

Workflow-bestand: `.github/workflows/performance-gate.yml`

## Scenario's beheren
Scenario-definitie staat in `perf/scenarios.ron`.

Velden:
- `name`: unieke scenarionaam
- `entities`: aantal gesimuleerde entities
- `steps`: aantal updates per meting
- `complexity`: hoeveelheid werk per entity per step

## Lokaal draaien
Gebruik:

```bash
cargo run --release --bin perf_scenarios -- \
  --scenario-file perf/scenarios.ron \
  --output perf_local.csv \
  --warmup 1 \
  --repeats 3
```

Vergelijk twee runs:

```bash
uv run scripts/compare_perf_results.py \
  --base perf_base.csv \
  --head perf_head.csv \
  --max-regression-pct 10
```

## Belangrijk voor GitHub instellingen
Om deze gate merge-blocking te maken:
1. Ga naar `Settings` -> `Branches`.
2. Open je branch protection rule voor `main`.
3. Zet `Require status checks to pass before merging` aan.
4. Selecteer de check `Compare base vs head performance`.

## Threshold aanpassen
Pas de drempel aan in `.github/workflows/performance-gate.yml`:
- `--max-regression-pct 10`
