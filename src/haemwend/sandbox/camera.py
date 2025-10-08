"""Camera controller for the Panda3D sandbox."""

from __future__ import annotations

from dataclasses import dataclass
from math import sqrt
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

    def __init__(
        self,
        *,
        settings: CameraSettings | None = None,
        boundary_radius: float = 30.0,
        ground_height: float = 1.6,
    ) -> None:
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
        self._boundary_radius = max(boundary_radius, 0.0)
        self._ground_height = max(ground_height, 0.0)
        self._magnet_strength = 0.96
        self._outside_boundary = False
        self._boundary_ratio = 0.0

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

    # Constraint configuration --------------------------------------

    def configure_boundary(self, *, radius: float | None = None, ground_height: float | None = None) -> None:
        """Update soft boundary and ground constraints for the camera."""

        if radius is not None:
            self._boundary_radius = max(radius, 0.0)
        if ground_height is not None:
            self._ground_height = max(ground_height, 0.0)

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
            self._apply_constraints(base)
            return

        direction.normalize()
        speed = self.settings.move_speed
        if self._key_state["sprint"]:
            speed *= self.settings.sprint_multiplier

        proposed = base.camera.getPos() + direction * speed * dt
        constrained = self._constrain_position(proposed)
        base.camera.setPos(constrained)

    def _update_task(self, task: Task) -> int:
        dt = float(globalClock.getDt())
        self.update(dt)
        return task.cont

    def _apply_constraints(self, base: ShowBase) -> None:
        constrained = self._constrain_position(base.camera.getPos())
        base.camera.setPos(constrained)

    def _constrain_position(self, position: Vec3) -> Vec3:
        x = float(position.x)
        y = float(position.y)
        z = max(float(position.z), self._ground_height)

        radius = self._boundary_radius
        planar_distance = sqrt(x * x + y * y)
        if radius > 0:
            ratio = planar_distance / radius
            self._boundary_ratio = ratio
            self._outside_boundary = ratio > 1.0
        else:
            self._boundary_ratio = 0.0
            self._outside_boundary = False

        if radius > 0 and planar_distance > radius:
            scale = (radius * self._magnet_strength) / planar_distance
            x *= scale
            y *= scale

        return Vec3(x, y, z)

    # Diagnostics -----------------------------------------------------

    @property
    def boundary_radius(self) -> float:
        return self._boundary_radius

    @property
    def ground_height(self) -> float:
        return self._ground_height

    @property
    def is_outside_boundary(self) -> bool:
        return self._outside_boundary

    @property
    def boundary_ratio(self) -> float:
        return self._boundary_ratio
