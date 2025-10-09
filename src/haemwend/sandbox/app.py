"""Panda3D application wrapper for the sandbox feature."""

from __future__ import annotations

from collections.abc import Callable
from typing import Any, Final

from direct.showbase.ShowBase import ShowBase  # type: ignore[import-not-found]
from panda3d.core import Vec4, WindowProperties  # type: ignore[import-not-found]

TaskCallback = Callable[[Any], int]

__all__ = ["SandboxApp"]


class SandboxApp(ShowBase):
    """Manage the Panda3D ShowBase lifecycle for the sandbox."""

    NAME: Final[str] = "haemwend.sandbox"

    def __init__(self, *, title: str = "Haemwend Sandbox") -> None:
        super().__init__()
        self._title = title
        self._running = False
        self._task_names: set[str] = set()

        self.disableMouse()
        self._configure_window()
        self._configure_scene()
        self.accept("escape", self.userExit)

    def _configure_window(self) -> None:
        window = getattr(self, "win", None)
        if window is None or not hasattr(window, "requestProperties"):
            return

        props = WindowProperties()
        props.setTitle(self._title)
        props.setSize(1280, 720)
        window.requestProperties(props)

    def _configure_scene(self) -> None:
        self.setBackgroundColor(Vec4(0.03, 0.04, 0.05, 1.0))

    def register_update_task(self, callback: TaskCallback, *, name: str) -> None:
        """Register a Panda3D task that should run each frame."""

        if name in self._task_names:
            self.taskMgr.remove(name)
        self._task_names.add(name)
        self.taskMgr.add(callback, name)

    def remove_update_task(self, name: str) -> None:
        """Remove a previously registered task."""

        if name in self._task_names:
            self.taskMgr.remove(name)
            self._task_names.discard(name)

    def start(self) -> None:
        """Start the Panda3D main loop."""

        if self._running:
            return
        self._running = True
        self.run()

    def stop(self) -> None:
        """Stop the Panda3D main loop."""

        if not self._running:
            return

        for name in list(self._task_names):
            self.remove_update_task(name)

        self._running = False
        self.userExit()

    @property
    def running(self) -> bool:
        """Whether the Panda3D app is currently running."""

        return self._running
