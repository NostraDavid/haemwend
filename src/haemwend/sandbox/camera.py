"""Camera controller for the Panda3D sandbox."""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING, Any

from direct.showbase.ShowBaseGlobal import globalClock  # type: ignore[import-not-found]
from panda3d.core import Vec3, WindowProperties  # type: ignore[import-not-found]

if TYPE_CHECKING:
    from direct.showbase.ShowBase import ShowBase  # type: ignore[import-not-found]
    from direct.task import Task  # type: ignore[import-not-found]
else:  # pragma: no cover - runtime fallback for type hints
    ShowBase = Any  # type: ignore[misc,assignment]
    Task = Any  # type: ignore[misc,assignment]


@dataclass(slots=True)
class CameraSettings:
    """Configuration inputs for the sandbox camera controller."""

    move_speed: float = 5.5
    sprint_multiplier: float = 1.8
    mouse_sensitivity: float = 0.15
    vertical_look_limit: float = 82.0


class SandboxCameraController:
    """First-person camera controller handling keyboard and mouse input."""

    _TASK_NAME = "sandbox-camera-update"

    def __init__(self, settings: CameraSettings | None = None) -> None:
        self.settings = settings or CameraSettings()
        self._base: ShowBase | None = None
        self._key_state: dict[str, bool] = {
            "forward": False,
            "back": False,
            "left": False,
            "right": False,
            "up": False,
            "down": False,
            "sprint": False,
        }
        self._event_bindings: list[tuple[str, bool]] = []
        self._window_center: tuple[int, int] | None = None
        self._yaw = 0.0
        self._pitch = 0.0

    def bind(self, base: ShowBase) -> None:
        """Attach the camera controller to Panda3D input callbacks."""

        if self._base is base:
            return
        if self._base is not None:
            self.unbind()

        self._base = base
        base.disableMouse()
        self._setup_window()
        self._yaw, self._pitch, _ = base.camera.getHpr()
        self._register_input_events(base)
        base.taskMgr.add(self._update_task, self._TASK_NAME)

    def unbind(self) -> None:
        """Detach the controller from Panda3D callbacks and tasks."""

        if self._base is None:
            return

        base = self._base
        for event, state in self._event_bindings:
            base.ignore(event if state else f"{event}-up")
        self._event_bindings.clear()

        base.taskMgr.remove(self._TASK_NAME)

        props = WindowProperties()
        props.setCursorHidden(False)
        base.win.requestProperties(props)

        self._base = None
        self._window_center = None

    def update(self, dt: float) -> None:
        """Drive camera movement based on elapsed time."""

        if self._base is None or dt <= 0:
            return

        base = self._base
        self._update_mouse_look(base)
        self._update_movement(base, dt)

    # Internal helpers -------------------------------------------------

    def _setup_window(self) -> None:
        assert self._base is not None
        window = self._base.win
        if window is None:
            return

        width = window.getXSize()
        height = window.getYSize()
        self._window_center = (width // 2, height // 2)

        props = WindowProperties()
        props.setCursorHidden(True)
        self._base.win.requestProperties(props)
        window.movePointer(0, *self._window_center)

    def _register_input_events(self, base: ShowBase) -> None:
        bindings = {
            "w": "forward",
            "s": "back",
            "a": "left",
            "d": "right",
            "space": "up",
            "control": "down",
            "shift": "sprint",
        }
        for key, action in bindings.items():
            base.accept(key, self._set_key_state, [action, True])
            base.accept(f"{key}-up", self._set_key_state, [action, False])
            self._event_bindings.append((key, True))
            self._event_bindings.append((key, False))

    def _set_key_state(self, action: str, pressed: bool) -> None:
        self._key_state[action] = pressed

    def _update_mouse_look(self, base: ShowBase) -> None:
        if self._window_center is None or base.win is None or not base.mouseWatcherNode.hasMouse():
            return

        pointer = base.win.getPointer(0)
        center_x, center_y = self._window_center
        delta_x = pointer.getX() - center_x
        delta_y = pointer.getY() - center_y

        if delta_x == 0 and delta_y == 0:
            return

        sensitivity = self.settings.mouse_sensitivity
        self._yaw -= delta_x * sensitivity
        self._pitch = max(
            -self.settings.vertical_look_limit,
            min(self.settings.vertical_look_limit, self._pitch - delta_y * sensitivity),
        )

        base.camera.setHpr(self._yaw, self._pitch, 0.0)
        base.win.movePointer(0, center_x, center_y)

    def _update_movement(self, base: ShowBase, dt: float) -> None:
        direction = Vec3(0, 0, 0)
        camera_quat = base.camera.getQuat(base.render)

        if self._key_state["forward"]:
            direction += camera_quat.getForward()
        if self._key_state["back"]:
            direction -= camera_quat.getForward()
        if self._key_state["left"]:
            direction -= camera_quat.getRight()
        if self._key_state["right"]:
            direction += camera_quat.getRight()
        if self._key_state["up"]:
            direction += camera_quat.getUp()
        if self._key_state["down"]:
            direction -= camera_quat.getUp()

        if direction.lengthSquared() == 0:
            return

        direction.normalize()
        speed = self.settings.move_speed
        if self._key_state["sprint"]:
            speed *= self.settings.sprint_multiplier

        base.camera.setPos(base.camera.getPos() + direction * speed * dt)

    def _update_task(self, task: Task) -> int:
        dt = float(globalClock.getDt())
        self.update(dt)
        return task.cont
