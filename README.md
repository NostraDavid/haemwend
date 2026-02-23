# Haemwend

Realtime Bevy-project met een Blender AI asset-workflow voor stylized modellen.

## Vereisten

- `nix` (aanbevolen) of een lokale Rust-toolchain
- `cargo`, `just`, `cargo-watch`
- `blender` (CLI beschikbaar in `PATH`)
- `git-lfs` (voor `.glb` en `.png`)

## Snel starten

```bash
nix develop
just run
```

Hot-reload workflow:

```bash
nix develop
just run-hot
```

- Codewijzigingen: auto-restart via `cargo watch`
- Assetwijzigingen: live reload via Bevy `file_watcher`

Zie ook `docs/hot-reload.md`.

## Belangrijke commando's

- `just run`: start de game
- `just run-hot`: game met auto-restart + asset hot-reload
- `just check`: compile-check
- `just test`: tests
- `just fmt`: formatteren
- `just ci`: check + fmt-check + clippy + test

## Blender AI workflow

Assets en scripts staan onder `assets/blender_ai/`.

- `just blender-ui`: start de Blender AI editor
- `just blender-ui-hot`: editor met hot restart bij code/config wijzigingen
- `just blender-table-validate`: valideer `table.py`
- `just blender-table-artifacts`: render + JSON report met timestamp
- `just blender-export assets/blender_ai/table.py`: exporteer model naar GLB

### UI controls (Blender AI editor)

- `RMB`: orbit/roteren
- `MMB`: pannen
- `Mouse wheel`: zoomen
- `Center View`: model opnieuw centreren/in frame zetten
- `Show grid`: toon/verberg meter-grid en assen

### Parameters opslaan/laden

In de editor:

- `Save Params`: sla huidige model-parameters op
- `Load Params`: laad opgeslagen parameters voor het actieve model
- `Reset Defaults`: terug naar defaults uit `config/blender_ai_models.ron`

Persistente presets staan in:

- `assets/blender_ai/_artifacts/editor_presets.ron`

## GLB in Bevy

- Exporteer naar `assets/models/*.glb`
- Gebruik het pad vervolgens direct in Bevy asset-loading
- `.glb` en `.png` worden via Git LFS beheerd (`.gitattributes`)

## Extra documentatie

- `assets/blender_ai/README.md`: Blender assetpipeline
- `assets/blender_ai/AGENT.md`: stijlregels en bronnen voor stylized assets
