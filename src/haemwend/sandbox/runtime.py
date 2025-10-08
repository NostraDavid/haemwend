"""Runtime coordinator for the Panda3D sandbox feature."""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING, Any, Final

from haemwend.infrastructure.config_loader import (
    DEFAULT_CONFIG_PATH,
    SandboxConfig,
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

        self._config = load_sandbox_config(self._config_path)
        self._logger.info(
            "sandbox.config.reloaded",
            primitives=len(self._config.environment.primitives),
            boundary_radius=self._config.environment.boundary_radius,
        )
        return self._config

    def launch(self) -> bool:
        """Launch the Panda3D sandbox if enabled and not already running."""

        if self._running:
            self._logger.warning("sandbox.already_running")
            return True

        config = self.config
        if not config.enabled:
            self._logger.info("sandbox.disabled")
            return False

        self._logger.info(
            "sandbox.starting",
            primitives=len(config.environment.primitives),
            boundary_radius=config.environment.boundary_radius,
        )

        app = SandboxApp()
        self._app = app

        camera_settings = CameraSettings(
            move_speed=config.camera.move_speed,
            sprint_multiplier=config.camera.sprint_multiplier,
            mouse_sensitivity=config.camera.mouse_sensitivity,
            vertical_look_limit=config.camera.vertical_look_limit,
        )
        camera = SandboxCameraController(
            settings=camera_settings,
            boundary_radius=config.environment.boundary_radius,
            ground_height=1.6,
        )
        camera.bind(app)
        camera.configure_boundary(radius=config.environment.boundary_radius, ground_height=1.6)
        self._camera = camera

        ui = SandboxUI()
        ui.bind(app)
        self._ui = ui

        self._register_runtime_bindings(app)
        self._register_feedback_task(app)
        ui.update_boundary_feedback(0.0)

        build_environment(app, config=config)

        self._running = True
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
            self._running = False
            self._logger.info("sandbox.stopped")
        return True

    def shutdown(self) -> None:
        """Gracefully stop the Panda3D sandbox if it is running."""

        if not self._running:
            self._logger.debug("sandbox.not_running")
            return

        self._logger.info("sandbox.stopping")

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
