# Tasks: Panda3D Walkable Sandbox Setup

**Input**: Design documents from `/specs/001-create-an-initial/`
**Prerequisites**: plan.md (required), spec.md (required)

**Tests**: Not requested for this feature; no test tasks are scheduled unless priorities change.

**Organization**: Tasks are grouped by user story so each increment stays independently implementable and demoable.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: The task can run in parallel with other [P] items (different files, no dependencies).
- **[Story]**: Story label for traceability (US1, US2, US3). Setup/Foundational/Polish items use descriptive tags.
- Explicit file paths keep implementation unambiguous.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Prepare dependencies, configuration surfaces, and package scaffolding required across stories.

- [X] T001 [Setup] Update `pyproject.toml` and regenerate `uv.lock` to add runtime dependencies `structlog` and `panda3d`.
- [X] T002 [P] [Setup] Create `config/sandbox.toml` with default enable toggle, camera tuning (speed, sensitivity), boundary radius (30m), and three primitive shape entries.
- [X] T003 [P] [Setup] Scaffold `src/haemwend/sandbox/` package with `__init__.py`, `app.py`, `environment.py`, `camera.py`, `ui.py`, and `runtime.py` placeholders.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Establish configuration and logging infrastructure that every story relies on.

- [X] T004 [Foundation] Implement `src/haemwend/infrastructure/config_loader.py` to parse `config/sandbox.toml` into dataclasses with validation and sensible defaults.
- [X] T005 [P] [Foundation] Create `src/haemwend/infrastructure/logging.py` to configure `structlog` for JSON output to stdout and expose a helper for sandbox modules.
- [X] T006 [Foundation] Flesh out `src/haemwend/sandbox/runtime.py` with a `SandboxRunner` skeleton that wires config loading, logging helpers, and the feature enable guard (no Panda3D logic yet).

**Checkpoint**: Configuration + logging stack ready; user story work can now safely begin.

---

## Phase 3: User Story 1 – Launch the sandbox and explore (Priority: P1) 🎯

**Goal**: Launch a Panda3D sandbox that opens quickly, displays ground + primitives, and supports free camera navigation.

**Independent Test**: From a clean checkout, run the sandbox entry point, reach the scene within 5 seconds, and traverse to each primitive without errors.

- [X] T007 [US1] Implement `SandboxApp` in `src/haemwend/sandbox/app.py` to initialize Panda3D `ShowBase`, set window properties, and manage the main loop.
- [X] T008 [P] [US1] Build environment primitives in `src/haemwend/sandbox/environment.py` to spawn a ground plane and at least three visible shapes with default transforms.
- [X] T009 [US1] Develop the first-person camera controller in `src/haemwend/sandbox/camera.py` with keyboard move/sprint, mouse look, and vertical rotation clamping.
- [X] T010 [US1] Connect `SandboxRunner` in `src/haemwend/sandbox/runtime.py` and update `src/haemwend/main.py` to launch the Panda3D app when enabled, logging structured start/stop events.

**Checkpoint**: User Story 1 delivers a playable sandbox slice suitable for MVP demos.

---

## Phase 4: User Story 2 – Understand controls and boundaries (Priority: P2)

**Goal**: Provide discoverable control guidance and enforce the circular walk boundary.

**Independent Test**: Launch the sandbox, toggle instructions with the `H` key, and confirm the camera remains above the ground plane and inside the 30 m radius boundary during movement.

- [X] T011 [P] [US2] Implement control overlay UI in `src/haemwend/sandbox/ui.py` that displays movement/look/sprint/help hints and toggles via the `H` key.
- [X] T012 [US2] Extend `src/haemwend/sandbox/camera.py` to enforce the 30 m radius boundary with gentle redirection and maintain ground-height constraints.
- [X] T013 [US2] Integrate the overlay and boundary feedback within `src/haemwend/sandbox/runtime.py`, wiring key listeners and ensuring instructions don’t obstruct exploration.

**Checkpoint**: User Stories 1 & 2 operate independently with discoverable controls and safe navigation.

---

## Phase 5: User Story 3 – Configure the sandbox for demos (Priority: P3)

**Goal**: Allow maintainers to tune camera behavior and shape layout through the TOML configuration without code edits.

**Independent Test**: Edit `config/sandbox.toml`, restart the sandbox, and confirm camera sensitivity, movement speeds, and shape layout reflect the new configuration while logging overrides.

- [ ] T014 [US3] Expand `src/haemwend/infrastructure/config_loader.py` to map camera sensitivity, movement speeds, shape catalog, and enable toggle overrides with validation against bounds.
- [ ] T015 [P] [US3] Update `src/haemwend/sandbox/environment.py` to instantiate primitives from config definitions and clamp positions inside the circular boundary.
- [ ] T016 [US3] Apply configurable camera parameters and update help text in `src/haemwend/sandbox/camera.py` and `ui.py`, ensuring runtime refresh on launch.
- [ ] T017 [US3] Emit structured override summaries and sandbox start/stop metadata from `src/haemwend/sandbox/runtime.py` using the logging helper.

**Checkpoint**: All three user stories are independently demoable, with configuration-driven tuning and telemetry in place.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final documentation and validation touches that raise confidence across the feature.

- [ ] T018 [Polish] Update `README.md` with Panda3D installation notes, sandbox launch instructions, control mapping, and configuration guidance.
- [ ] T019 [P] [Polish] Author `specs/001-create-an-initial/quickstart.md` summarizing manual verification steps for each user story and listing known edge cases.

---

## Dependencies & Execution Order

### Phase Dependencies

1. **Setup** → 2. **Foundational** → 3. **User Story 1 (P1)** → 4. **User Story 2 (P2)** → 5. **User Story 3 (P3)** → 6. **Polish**

### User Story Dependencies

- **US1** depends on Foundational tasks completing.
- **US2** depends on US1 (needs working sandbox and camera base).
- **US3** depends on US1 (environment/camera code) and leverages US2’s overlay for updated messaging but remains independently testable once prior phases finish.

### Task-Level Notes

- Tasks touching the same file remain sequential to avoid merge conflicts.
- [P] tasks operate on distinct files or assets and can proceed in parallel once their prerequisites finish.

## Parallel Execution Examples

- **Setup**: Run T002 and T003 in parallel after T001 adds dependencies.
- **US1**: T008 can progress alongside T007 once the app skeleton exists; integrate both before executing T010.
- **US2**: T011 (UI overlay) can start while T012 tightens boundary logic; both feed into T013.
- **US3**: T015 (environment from config) can proceed in parallel with T014 once new dataclasses exist.

## Implementation Strategy

1. Complete Setup and Foundational phases to lock in dependencies, configuration, and logging.
2. Deliver **MVP** by finishing User Story 1 — this enables early walkthroughs.
3. Layer User Story 2 for discoverability and safety improvements.
4. Add User Story 3 to unlock configuration-driven demos and telemetry.
5. Close with polish tasks to document usage and validation steps.

Stopping after User Story 1 still yields a usable MVP; subsequent stories progressively enhance onboarding and configurability.
