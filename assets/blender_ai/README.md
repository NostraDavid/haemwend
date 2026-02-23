# Blender AI Assets

Scripts in deze map genereren stylized Blender assets (nu: `table.py`) met reproduceerbare validatie en render-artifacts.

## Snel starten

1. Valideer geometrie:
   - `just blender-table-validate`
2. Genereer timestamped artifacts:
   - `just blender-table-artifacts`
3. Open live parameter UI:
   - `just blender-ui`

Artifacts komen in `assets/blender_ai/_artifacts/` en worden door git genegeerd.

## Bestanden

- `table.py`: assetgenerator voor low-poly tafel.
- `blender_utils.py`: gedeelde Blender helpers.
- `AGENT.md`: stijlrichtlijnen, heuristieken en bronnen.

## Stijlrichtlijnen

Volg de richtlijnen in [`AGENT.md`](AGENT.md) voor non-boxy vormen, readability op afstand en stylized exaggeratie.
