# Haemwend

Early experiments for the Haemwend world are bundled into a Panda3D-powered sandbox. It spins up a small walkable scene, wired to structured logging and runtime configuration so designers can iterate without touching code.

## Prerequisites

- Python 3.13 (the project targets the latest CPython builds)
- [uv](https://docs.astral.sh/uv/) 0.8 or newer for dependency management *(recommended)*
- Graphics drivers capable of running OpenGL 3.0 or above

### Panda3D installation notes

`uv sync` (or `pip install haemwend`) will pull the `panda3d` wheel automatically. The wheels include the runtime engine on macOS, Windows, and most mainstream Linux distributions. If the wheel is unavailable for your platform:

- Install Panda3D from the [official downloads](https://www.panda3d.org/download.php) page, then re-run `uv sync` so Python bindings are available.
- Linux packages may require system libraries such as `libegl1` and `libgl1`. Install them through your distro package manager when the sandbox reports missing shared objects.

## Setup

```bash
uv sync
```

The command creates a virtual environment at `.venv/` and installs both runtime and development dependencies declared in `pyproject.toml`. If you prefer `pip`, run `python -m venv .venv && source .venv/bin/activate` followed by `pip install -e .[dev]`.

## Launching the sandbox

1. Confirm the feature flag in `config/sandbox.toml` is enabled:

   ```toml
   enabled = true
   ```

2. Start the interactive scene:

   ```bash
   uv run haemwend
   ```

   Alternative entry points:

   - `uv run python -m haemwend`
   - `python -m haemwend` after activating `.venv`

3. A Panda3D window opens within a second or two. Structured logs stream to stdout in JSON format; capture them for later analysis if desired.

Shut down the sandbox at any time with `Esc` or `Ctrl+C` in the launching terminal.

## Controls at a glance

The instructional overlay appears automatically. Toggle it with `H` after you memorise the mapping.

- `W / A / S / D` — walk (~5.5 m/s by default)
- Mouse — look around (sensitivity 0.15)
- `Shift` — sprint multiplier (×1.8)
- `Space` / `Ctrl` — rise or descend while flying
- `H` — toggle help overlay (also shows boundary hints)
- `Esc` — close the window

Boundary feedback shows as amber text when you are within 10 % of the perimeter and red text when redirects are being applied.

## Configuration guide

All tunables live in `config/sandbox.toml`. Changes take effect on the next launch.

| Section | Keys | Notes |
| --- | --- | --- |
| Root | `enabled` | Set to `false` to disable the sandbox without removing the package. |
| `[camera]` | `move_speed`, `sprint_multiplier`, `mouse_sensitivity`, `vertical_look_limit` | Values are validated against sensible bounds before applying to the runtime. Use them to adjust traversal feel. |
| `[environment]` | `boundary_radius` | Defines the circular walk boundary in metres. Camera motion gently redirects players who cross it. |
| `[[environment.primitives]]` | `name`, `type`, `position`, `scale`, `color` | Each entry spawns a primitive (cube, sphere, or cylinder). Positions are clamped inside the boundary and rendered on a shared ground plane. |

Whenever overrides differ from the shipped defaults, the runtime emits a `sandbox.config.overrides` log with the delta so you can audit tweaks made for demos.

## Troubleshooting tips

- **No Panda3D window**: The runtime now retries with Panda3D's software renderer when the default graphics pipe fails. Look for a `sandbox.display.fallback` log event that lists the selected backend (e.g. `p3tinydisplay`). If a subsequent `sandbox.display.unavailable` event appears, install OpenGL libraries such as `libgl1` / `mesa-libGL` or provision a virtual framebuffer.
- **Missing shared object errors**: Install OpenGL/EGL libraries and verify the `panda3d` wheel is compatible with your architecture.
- **Controls feel off**: Adjust `[camera]` settings; logs confirm the applied values on launch and reload.

For manual verification steps and known edge cases, see `specs/001-create-an-initial/quickstart.md`.
