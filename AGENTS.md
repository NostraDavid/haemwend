# Repository Guidelines

## Project Structure & Module Organization
This is a Godot 4.5 project. The root contains the project manifest and a minimal set of assets:
- `project.godot`: Godot project configuration (edit via the editor UI).
- `icon.svg`: Project icon used by Godot.

Add game content under `res://` using standard Godot folders. Recommended layout:
- `scenes/` for `.tscn` and `.scn` files
- `scripts/` for `.gd` scripts
- `assets/` for art, audio, and other media

## Build, Test, and Development Commands
Use the Godot editor and CLI for local development:
- `godot --editor --path .` opens the project in the editor.
- `godot --path .` runs the current project.

If you use a different Godot binary name, substitute it (e.g., `Godot_v4.5-stable`).

## Coding Style & Naming Conventions
Follow Godot/GDScript conventions:
- Indentation: 4 spaces (no tabs).
- Scripts: `snake_case.gd` (e.g., `player_controller.gd`).
- Scenes: `PascalCase.tscn` (e.g., `Player.tscn`).
- Nodes: `PascalCase` names in the scene tree.

Format via the Godot editor’s built-in formatter where possible.

## Testing Guidelines
No automated tests are currently configured. If you add tests, document:
- Framework choice (e.g., GUT).
- Where tests live (e.g., `tests/`).
- How to run them.

## Commit & Pull Request Guidelines
Recent commits use short, sentence-style messages (e.g., “add flake.nix and fix startup”). Keep messages:
- In present tense
- Under ~72 characters
- Focused on one change

For pull requests:
- Include a clear description of behavior changes.
- Link related issues (if any).
- Add screenshots or clips for visual changes.

## Configuration Tips
Do not hand-edit `project.godot` unless necessary; prefer the editor to avoid format drift.
