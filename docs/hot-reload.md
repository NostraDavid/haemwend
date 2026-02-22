# Hot-reload workflow

Voor dit project is de praktische workflow:
- Codewijzigingen: auto-restart via `cargo watch`.
- Assetwijzigingen: live hot-reload via Bevy `file_watcher`.

## Starten

```bash
nix develop
./scripts/dev_game.sh
```

Wat dit script doet:
- Start `cargo watch` op `src/`, `Cargo.toml`, `Cargo.lock` (en `assets/` als die map bestaat).
- Herstart de game automatisch bij codewijzigingen.
- Start de game met `--features bevy/file_watcher` voor asset hot-reload.

## Beperkingen
- Rust code wordt niet live in een draaiend proces gepatcht; de game herstart bij codewijziging.
- Assets kunnen tijdens runtime wel opnieuw geladen worden.

## Handmatig (zonder script)

```bash
cargo watch -w src -w Cargo.toml -x "run --features bevy/file_watcher --bin haemwend"
```
