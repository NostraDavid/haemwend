# Panda3D Sandbox Quickstart

This guide condenses the manual checks for the Haemwend sandbox feature delivered across User Stories 1–3. Follow the scenarios below whenever you need to validate a fresh build or demo configuration.

## Environment checklist

1. Install Python 3.13 and ensure `uv` 0.8+ is available on your PATH.
2. From the repository root, install dependencies:

   ```bash
   uv sync
   ```

3. Verify Panda3D launched successfully in the past on the target machine. If the engine fails to open a window, revisit GPU driver and OpenGL/EGL library requirements before proceeding.
4. Confirm `config/sandbox.toml` exists. Copy it from `config/sandbox.toml` in the repo if needed.

## User Story 1 — Launch the sandbox and explore

### Steps — launch & explore

1. Ensure `config/sandbox.toml` has `enabled = true`.
2. Start the sandbox:

   ```bash
   uv run haemwend
   ```

3. Within five seconds a Panda3D window should appear with a flat ground plane and three coloured primitives (obelisk, pillar, orb).
4. Use `W / A / S / D` to walk and the mouse to look around. Hold `Shift` to confirm sprinting increases speed. Tap `Space` and `Ctrl` to verify vertical movement.
5. Close the window with `Esc` or `Ctrl+C` in the terminal.

### Pass criteria — launch & explore

- Startup logs include `sandbox.starting` and list primitive names.
- Window responds to mouse-look immediately without raw mouse cursor visible.
- Shutdown logs include `sandbox.stopped` with a duration.

Capture any deviations or crashes, including stack traces and `sandbox.*` log lines.

## User Story 2 — Understand controls and boundaries

### Steps — controls & boundary

1. Launch the sandbox as above.
2. Observe that the instructional overlay appears in the top-right corner listing controls.
3. Press `H` to hide the overlay, then `H` again to reveal it.
4. Walk toward the edge of the scene (≈30 metres from centre). Watch the overlay for amber “Approaching perimeter” messaging, then red “Boundary reached” text once you exceed the limit.
5. Continue holding `W` outside the perimeter and confirm the camera is gently redirected back inside the boundary and never sinks below the ground plane.

### Pass criteria — controls & boundary

- Overlay toggles instantly with `H` and remembers visibility while running.
- Boundary feedback messages show at ~90 % radius and when past the limit.
- Camera position clamps to a minimum height of 1.6 metres and smoothly redirects toward the playable area when outside the boundary.

## User Story 3 — Configure the sandbox for demos

### Steps — configuration tuning

1. Edit `config/sandbox.toml` to adjust camera behaviour, for example:

   ```toml
   [camera]
   move_speed = 8.0
   sprint_multiplier = 2.2
   mouse_sensitivity = 0.2
   vertical_look_limit = 75.0
   ```

2. Add or tweak a primitive entry, ensuring it stays within the boundary:

   ```toml
   [[environment.primitives]]
   name = "south_marker"
   type = "cube"
   position = [0.0, -25.0, 0.0]
   scale = [1.5, 1.5, 3.0]
   color = [0.2, 0.6, 0.9, 1.0]
   ```

3. Relaunch the sandbox (`uv run haemwend`).
4. Check the overlay text for updated speed/sensitivity values. Sprint and mouse-look to verify the new feel.
5. Locate the adjusted or new primitive. If it was placed outside the 30 m boundary, confirm it has been clamped onto the perimeter and the environment log notes `clamped = true`.
6. Review stdout for `sandbox.config.overrides` and `sandbox.environment.primitive` log entries summarising the applied overrides.

### Pass criteria — configuration tuning

- Camera movement matches edited speeds and overlay text reports the new values.
- The scene contains the configured primitive(s); out-of-bounds entries are clamped inside the boundary and logged as such.
- No `SandboxConfigError` exceptions arise when using values within documented bounds.

## Known edge cases and tips

- **Feature flag disabled**: Setting `enabled = false` skips Panda3D startup and logs `sandbox.disabled`. Re-enable the flag for manual verification runs.
- **Invalid configuration values**: Speeds, sensitivity, and boundary radius have defined min/max limits. Out-of-range values raise `SandboxConfigError` during launch; fix the TOML and retry.
- **Primitive geometry**: All primitives currently render as card-based meshes. Non-standard types (not one of cube, cylinder, sphere, card, plane) trigger a validation error.
- **Headless environments**: On CI or SSH sessions without a display, the Panda3D window cannot open. Use a virtual framebuffer (e.g., `xvfb-run`) if you must capture screenshots.
- **Mouse grab focus**: The camera hides the cursor and captures the pointer. Losing window focus can stall mouse-look; click back into the window to regain control.

Log artifacts (`sandbox.*` events) are valuable when sharing findings—attach them to bug reports together with reproduction steps above.
