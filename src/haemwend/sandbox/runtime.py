"""Runtime coordinator for the Panda3D sandbox feature."""

from __future__ import annotations

from pathlib import Path
from typing import Final

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

__all__ = ["SandboxRunner"]


class SandboxRunner:
    """Coordinate sandbox startup, configuration loading, and teardown."""

    _LOGGER_COMPONENT: Final[str] = "runtime"

    def __init__(self, config_path: Path | None = None) -> None:
        self._config_path = Path(config_path) if config_path is not None else DEFAULT_CONFIG_PATH
        self._config: SandboxConfig | None = None
        self._logger = get_logger(component=self._LOGGER_COMPONENT)
        self._running = False
        self._app: SandboxApp | None = None
        self._camera: SandboxCameraController | None = None
        self._ui: SandboxUI | None = None

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
        camera = SandboxCameraController(settings=camera_settings)
        camera.bind(app)
        self._camera = camera

        self._ui = SandboxUI()

        build_environment(app, config=config)

        self._running = True
        try:
            app.start()
        finally:
            if self._camera is not None:
                self._camera.unbind()
                self._camera = None
            self._app = None
            self._ui = None
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
            self._app.stop()
            self._app = None

        self._ui = None
        self._running = False

    def __repr__(self) -> str:  # pragma: no cover - helper for debugging
        return f"SandboxRunner(config_path={self._config_path!s}, running={self._running})"
