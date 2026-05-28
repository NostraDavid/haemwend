# Blender AI Assets

Scripts in deze map genereren stylized Blender assets (`table.py`, `tree_generator_addon.py`) met reproduceerbare validatie en render-artifacts.

## Snel starten

1. Valideer geometrie:
   - `just blender-table-validate`
   - `just blender-tree-validate`
2. Genereer timestamped artifacts:
   - `just blender-table-artifacts`
3. Open live parameter UI:
   - `just blender-ui`
4. Gebruik de boomgenerator in Blender:
   - Run `assets/blender_ai/tree_generator_addon.py` of installeer het script als add-on.
   - Open `View3D > Sidebar > LLM Trees`.
   - Parameterwijzigingen vervangen de huidige gegenereerde boom automatisch na een korte debounce-delay.

Artifacts komen in `assets/blender_ai/_artifacts/` en worden door git genegeerd.

## Bestanden

- `table.py`: assetgenerator voor low-poly tafel.
- `tree_generator_addon.py`: native Blender paneel en scriptbare generator voor stylized broadleaf/pine bomen.
- `blender_utils.py`: gedeelde Blender helpers.
- `AGENT.md`: stijlrichtlijnen, heuristieken en bronnen.

## Stijlrichtlijnen

Volg de richtlijnen in [`AGENT.md`](AGENT.md) voor non-boxy vormen, readability op afstand en stylized exaggeratie.
