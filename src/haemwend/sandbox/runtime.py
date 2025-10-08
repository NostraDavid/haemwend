"""Runtime coordinator for the Panda3D sandbox feature."""

from __future__ import annotations

from dataclasses import asdict
from pathlib import Path
from time import perf_counter
from typing import TYPE_CHECKING, Any, Final

from haemwend.infrastructure.config_loader import (
    DEFAULT_CONFIG_PATH,
    SandboxConfig,
    default_sandbox_config,
    load_sandbox_config,
)
from haemwend.infrastructure.logging import get_logger
from haemwend.sandbox.app import SandboxApp
from haemwend.sandbox.camera import CameraSettings, SandboxCameraController
from haemwend.sandbox.environment import build_environment
from haemwend.sandbox.ui import SandboxUI

if TYPE_CHECKING:
    from direct.task import Task  # type: ignore[import-not-found]
else:  # pragma: no cover - runtime fallback when Panda3D is unavailable
    Task = Any  # type: ignore[misc,assignment]

__all__ = ["SandboxRunner"]


class SandboxRunner:
    """Coordinate sandbox startup, configuration loading, and teardown."""

    _LOGGER_COMPONENT: Final[str] = "runtime"
    _FEEDBACK_TASK_NAME: Final[str] = "sandbox-ui-feedback"
    _GROUND_HEIGHT: Final[float] = 1.6

    def __init__(self, config_path: Path | None = None) -> None:
        self._config_path = Path(config_path) if config_path is not None else DEFAULT_CONFIG_PATH
        self._config: SandboxConfig | None = None
        self._logger = get_logger(component=self._LOGGER_COMPONENT)
        self._running = False
        self._app: SandboxApp | None = None
        self._camera: SandboxCameraController | None = None
        self._ui: SandboxUI | None = None
        self._event_bindings: list[str] = []
        self._task_handles: list[str] = []
        self._session_start: float | None = None

    @property
    def config_path(self) -> Path:
        """Return the path to the active sandbox configuration file."""

        return self._config_path

    @property
    def config(self) -> SandboxConfig:
        """Return the cached sandbox configuration, loading it if needed."""

        if self._config is None:
            self._config = load_sandbox_config(self._config_path)
        return self._config

    @property
    def running(self) -> bool:
        """Whether the sandbox runtime has been launched."""

        return self._running

    def reload_config(self) -> SandboxConfig:
        """Force a configuration reload from disk."""

        config = load_sandbox_config(self._config_path)
        self._config = config

        overrides = self._log_config_overrides(config, action="reload")

        self._logger.info(
            "sandbox.config.reloaded",
            enabled=config.enabled,
            overrides_applied=bool(overrides),
            camera={
                "move_speed": config.camera.move_speed,
                "sprint_multiplier": config.camera.sprint_multiplier,
                "mouse_sensitivity": config.camera.mouse_sensitivity,
                "vertical_look_limit": config.camera.vertical_look_limit,
            },
            environment={
                "boundary_radius": config.environment.boundary_radius,
                "primitive_count": len(config.environment.primitives),
            },
        )

        self._apply_config_to_runtime(config)
        return config

    def launch(self) -> bool:
        """Launch the Panda3D sandbox if enabled and not already running."""

        if self._running:
            self._logger.warning("sandbox.already_running")
            return True

        config = self.config
        overrides = self._log_config_overrides(config, action="launch")
        if not config.enabled:
            self._logger.info(
                "sandbox.disabled",
                overrides_applied=bool(overrides),
                config_path=str(self._config_path),
            )
            return False

        environment = config.environment
        camera_config = config.camera
        primitive_count = len(environment.primitives)
        primitive_names = [primitive.name for primitive in environment.primitives]

        self._logger.info(
            "sandbox.starting",
            config_path=str(self._config_path),
            overrides_applied=bool(overrides),
            camera={
                "move_speed": camera_config.move_speed,
                "sprint_multiplier": camera_config.sprint_multiplier,
                "mouse_sensitivity": camera_config.mouse_sensitivity,
                "vertical_look_limit": camera_config.vertical_look_limit,
            },
            environment={
                "boundary_radius": environment.boundary_radius,
                "primitive_count": primitive_count,
                "primitive_names": primitive_names,
            },
        )

        app = SandboxApp()
        self._app = app

        camera_settings = self._camera_settings_from_config(config)
        camera = SandboxCameraController(settings=camera_settings)
        camera.bind(app)
        self._camera = camera

        help_lines = self._format_help_lines(config)
        ui = SandboxUI(help_lines=help_lines)
        ui.bind(app)
        self._ui = ui

        ui.update_boundary_feedback(0.0)

        self._register_runtime_bindings(app)
        self._register_feedback_task(app)

        build_environment(app, config=config)

        self._running = True
        self._session_start = perf_counter()
        try:
            app.start()
        finally:
            self._cleanup_runtime_bindings()
            if self._camera is not None:
                self._camera.unbind()
                self._camera = None
            if self._ui is not None:
                self._ui.unbind()
                self._ui = None
            self._app = None
            duration = None
            if self._session_start is not None:
                duration = perf_counter() - self._session_start
            self._session_start = None
            self._running = False
            self._logger.info(
                "sandbox.stopped",
                duration_seconds=duration,
                primitive_count=primitive_count,
                overrides_applied=bool(overrides),
            )
        return True

    def shutdown(self) -> None:
        """Gracefully stop the Panda3D sandbox if it is running."""

        if not self._running:
            self._logger.debug("sandbox.not_running")
            return

        overrides_active = bool(self._summarize_config_overrides(self.config))
        self._logger.info(
            "sandbox.stopping",
            trigger="shutdown",
            overrides_applied=overrides_active,
            config_path=str(self._config_path),
        )

        if self._camera is not None:
            self._camera.unbind()
            self._camera = None

        if self._app is not None:
            self._cleanup_runtime_bindings()
            self._app.stop()
            self._app = None

        self._ui = None
        self._running = False

    def __repr__(self) -> str:  # pragma: no cover - helper for debugging
        return f"SandboxRunner(config_path={self._config_path!s}, running={self._running})"

    # Internal helpers -------------------------------------------------

    def _register_runtime_bindings(self, app: SandboxApp) -> None:
        events = ("h", "H")
        for event in events:
            app.accept(event, self._handle_help_toggle)
            self._event_bindings.append(event)

    def _register_feedback_task(self, app: SandboxApp) -> None:
        handle = app.taskMgr.add(self._update_feedback_task, self._FEEDBACK_TASK_NAME)
        if handle is not None:
            self._task_handles.append(self._FEEDBACK_TASK_NAME)

    def _cleanup_runtime_bindings(self) -> None:
        app = self._app
        if app is None:
            self._event_bindings.clear()
            self._task_handles.clear()
            return

        for event in self._event_bindings:
            app.ignore(event)
        self._event_bindings.clear()

        for task_name in self._task_handles:
            app.taskMgr.remove(task_name)
        self._task_handles.clear()

    def _handle_help_toggle(self) -> None:
        if self._ui is None:
            return
        self._ui.toggle()
        self._logger.info("sandbox.help.toggled", visible=self._ui.visible)

    def _update_feedback_task(self, task: Task) -> int:
        camera = self._camera
        ui = self._ui
        if camera is not None and ui is not None:
            ui.update_boundary_feedback(camera.boundary_ratio)
        return task.cont

    def _apply_config_to_runtime(self, config: SandboxConfig) -> None:
        """Propagate configuration changes to runtime components when available."""

        camera_settings = self._camera_settings_from_config(config)

        camera = self._camera
        if camera is not None:
            camera.apply_settings(camera_settings)

        ui = self._ui
        if ui is not None and (self._running or self._app is not None):
            ui.set_control_instructions(self._format_help_lines(config))

    def _camera_settings_from_config(self, config: SandboxConfig) -> CameraSettings:
        environment = config.environment
        camera = config.camera
        return CameraSettings(
            move_speed=camera.move_speed,
            sprint_multiplier=camera.sprint_multiplier,
            mouse_sensitivity=camera.mouse_sensitivity,
            vertical_look_limit=camera.vertical_look_limit,
            boundary_radius=environment.boundary_radius,
            ground_height=self._GROUND_HEIGHT,
        )

    def _log_config_overrides(self, config: SandboxConfig, *, action: str) -> dict[str, Any]:
        overrides = self._summarize_config_overrides(config)
        self._logger.info(
            "sandbox.config.overrides",
            action=action,
            overrides=overrides,
            using_defaults=not bool(overrides),
        )
        return overrides

    def _summarize_config_overrides(self, config: SandboxConfig) -> dict[str, Any]:
        defaults = asdict(default_sandbox_config())
        current = asdict(config)
        diff = self._diff_structures(defaults, current)
        if isinstance(diff, dict):
            return diff
        return {}

    def _diff_structures(self, default: Any, current: Any) -> Any | None:
        if isinstance(default, dict) and isinstance(current, dict):
            result: dict[str, Any] = {}
            for key, current_value in current.items():
                default_value = default.get(key)
                difference = self._diff_structures(default_value, current_value)
                if difference is not None:
                    result[key] = difference
            return result or None

        if isinstance(default, (list, tuple)) and isinstance(current, (list, tuple)):
            if list(current) != list(default):
                return current
            return None

        if default is None and current is not None:
            return current

        if current != default:
            return current

        return None

    def _format_help_lines(self, config: SandboxConfig) -> list[str]:
        camera = config.camera
        boundary_radius = config.environment.boundary_radius
        return [
            f"W / A / S / D — walk ({camera.move_speed:.1f} m/s)",
            f"Mouse — look (sensitivity {camera.mouse_sensitivity:.2f})",
            f"Shift — sprint (x{camera.sprint_multiplier:.1f})",
            "Space / Ctrl — rise & descend",
            f"H — toggle help | Boundary {boundary_radius:.0f} m",
            "Esc — quit window",
        ]
