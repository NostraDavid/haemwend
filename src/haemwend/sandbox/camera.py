"""Camera controller for the Panda3D sandbox."""

from __future__ import annotations

from dataclasses import dataclass
from math import cos, radians, sin
from typing import TYPE_CHECKING, Any

from panda3d.core import (  # type: ignore[import-not-found]
    BitMask32,
    CollisionHandlerQueue,
    CollisionNode,
    CollisionRay,
    CollisionTraverser,
    NodePath,
    Vec3,
    WindowProperties,
)

if TYPE_CHECKING:
    from direct.showbase.ShowBase import ShowBase  # type: ignore[import-not-found]
    from direct.task import Task  # type: ignore[import-not-found]

    from haemwend.sandbox.character import Character
else:
    ShowBase = Any
    Task = Any


@dataclass(slots=True)
class CameraSettings:
    """Configuration inputs for the sandbox camera controller."""

    distance: float = 10.0
    height: float = 2.0
    mouse_sensitivity: float = 0.2
    min_pitch: float = -20.0
    max_pitch: float = 60.0
    zoom_speed: float = 1.0
    min_distance: float = 2.0
    max_distance: float = 20.0


class SandboxCameraController:
    """Third-person camera controller handling mouse input."""

    _TASK_NAME = "sandbox-camera-update"

    def __init__(
        self,
        *,
        settings: CameraSettings | None = None,
    ) -> None:
        self.settings = settings or CameraSettings()
        self._base: ShowBase | None = None
        self._target: Character | None = None

        self._yaw = 0.0
        self._pitch = -20.0
        self._distance = self.settings.distance

        self._mouse_buttons: dict[str, bool] = {
            "left": False,
            "right": False,
            "middle": False,
        }
        self._window_center: tuple[int, int] | None = None
        self._last_mouse_pos: tuple[float, float] | None = None

        # Collision setup
        self._c_trav: CollisionTraverser | None = None
        self._c_queue: CollisionHandlerQueue | None = None
        self._c_ray_node: NodePath | None = None
        self._c_ray: CollisionRay | None = None

    def bind(self, base: ShowBase, target: Character) -> None:
        """Attach the camera controller to Panda3D input callbacks."""
        if self._base is base:
            return
        if self._base is not None:
            self.unbind()

        self._base = base
        self._target = target

        # Setup collision
        self._c_trav = CollisionTraverser()
        self._c_queue = CollisionHandlerQueue()

        self._c_ray = CollisionRay()
        c_node = CollisionNode("camera-ray")
        c_node.addSolid(self._c_ray)
        c_node.setFromCollideMask(BitMask32.bit(0))
        c_node.setIntoCollideMask(BitMask32.allOff())

        self._c_ray_node = base.render.attachNewNode(c_node)
        self._c_trav.addCollider(self._c_ray_node, self._c_queue)

        base.disableMouse()
        self._setup_window()

        # Register mouse events
        base.accept("mouse1", self._set_mouse_button, ["left", True])
        base.accept("mouse1-up", self._set_mouse_button, ["left", False])
        base.accept("mouse3", self._set_mouse_button, ["right", True])
        base.accept("mouse3-up", self._set_mouse_button, ["right", False])
        base.accept("wheel_up", self._zoom, [-1])
        base.accept("wheel_down", self._zoom, [1])

        base.taskMgr.add(self._update_task, self._TASK_NAME)

    def unbind(self) -> None:
        """Detach the controller from Panda3D callbacks and tasks."""
        if self._base is None:
            return

        base = self._base
        base.ignore("mouse1")
        base.ignore("mouse1-up")
        base.ignore("mouse3")
        base.ignore("mouse3-up")
        base.ignore("wheel_up")
        base.ignore("wheel_down")

        base.taskMgr.remove(self._TASK_NAME)

        if self._c_ray_node:
            self._c_ray_node.removeNode()
            self._c_ray_node = None
        self._c_ray = None
        self._c_queue = None
        self._c_trav = None

        props = WindowProperties()
        props.setCursorHidden(False)
        props.setMouseMode(WindowProperties.M_absolute)
        window = base.win
        if window is not None and hasattr(window, "requestProperties"):
            window.requestProperties(props)

        self._base = None
        self._target = None

    def _set_mouse_button(self, button: str, pressed: bool) -> None:
        self._mouse_buttons[button] = pressed

        if self._base:
            props = WindowProperties()
            if self._mouse_buttons["right"] or self._mouse_buttons["left"]:
                props.setCursorHidden(True)
                props.setMouseMode(WindowProperties.M_relative)

                # Center mouse immediately to avoid jump on first frame
                win = self._base.win
                if win:
                    width = win.getXSize()
                    height = win.getYSize()
                    win.movePointer(0, width // 2, height // 2)
            else:
                props.setCursorHidden(False)
                props.setMouseMode(WindowProperties.M_absolute)

            win = self._base.win
            if win:
                win.requestProperties(props)

    def _zoom(self, direction: int) -> None:
        self._distance += direction * self.settings.zoom_speed
        self._distance = max(self.settings.min_distance, min(self.settings.max_distance, self._distance))

    def _update_mouse_look(self, base: ShowBase) -> None:
        if not (self._mouse_buttons["right"] or self._mouse_buttons["left"]):
            return

        md = base.win.getPointer(0)
        md.getX()
        md.getY()

        if hasattr(base.win, "getXSize") and hasattr(base.win, "getYSize"):
            center_x = base.win.getXSize() // 2
            center_y = base.win.getYSize() // 2

            # In relative mode, we might need to handle delta differently depending on OS/Panda version
            # But typically with M_relative, we check delta from last frame or center reset
            # For simplicity with M_relative, Panda usually keeps pointer centered or gives deltas

            # Let's try using mouseWatcherNode for deltas if available, or manual calculation
            mouse_watcher = base.mouseWatcherNode
            if mouse_watcher.hasMouse():
                # This gives normalized coordinates (-1 to 1)
                mpos = mouse_watcher.getMouse()
                mpos.getX() * 100  # Scale up
                mpos.getY() * 100

                # Reset mouse to center to avoid hitting screen edges (if not in relative mode)
                # But we set M_relative, so we should rely on that if it works.
                # If M_relative works, the mouse stays put and we get deltas?
                # Actually, Panda3D M_relative behavior can be tricky.
                # Let's stick to the standard "re-center mouse" approach for robustness if M_relative isn't perfect.

        # Using raw mouse input from MouseWatcher is often better for cameras
        mw = base.mouseWatcherNode
        if mw.hasMouse():
            # We need deltas. Since we can't easily get deltas without resetting mouse,
            # let's assume M_relative works and gives us unbounded movement or we use the previous approach.
            # The previous code used movePointer to center. Let's reuse that logic but adapted.
            pass

        # Re-implementing the centering logic from previous camera controller as it's reliable
        window = base.win
        if window is None:
            return

        width = window.getXSize()
        height = window.getYSize()
        center_x = width // 2
        center_y = height // 2

        pointer = window.getPointer(0)
        current_x = pointer.getX()
        current_y = pointer.getY()

        delta_x = current_x - center_x
        delta_y = current_y - center_y

        if delta_x == 0 and delta_y == 0:
            return

        self._yaw += delta_x * self.settings.mouse_sensitivity
        self._pitch -= delta_y * self.settings.mouse_sensitivity
        self._pitch = max(self.settings.min_pitch, min(self.settings.max_pitch, self._pitch))

        window.movePointer(0, center_x, center_y)

        # If right click is held, rotate the character to match camera yaw
        if self._mouse_buttons["right"] and self._target:
            self._target.set_h(-self._yaw)

    def _update_task(self, task: Any) -> int:
        if self._base is None or self._target is None:
            return task.cont

        self._update_mouse_look(self._base)

        # Calculate camera position
        # Convert spherical coordinates to Cartesian
        # Pitch is rotation around X axis (up/down)
        # Yaw is rotation around Z axis (left/right)

        # If right click, camera yaw matches character yaw (mostly)
        # If left click, camera orbits freely (yaw changes, character doesn't)

        rad_pitch = radians(self._pitch)
        rad_yaw = radians(self._yaw)

        # Offset from target
        x = self._distance * cos(rad_pitch) * sin(-rad_yaw)
        y = self._distance * cos(rad_pitch) * cos(-rad_yaw)
        z = self._distance * sin(rad_pitch)

        target_pos = self._target.get_pos()
        # Add height offset to look at head/torso instead of feet
        target_center = target_pos + Vec3(0, 0, self.settings.height)

        # Collision check
        actual_distance = self._distance
        if self._c_trav and self._c_queue and self._c_ray and self._c_ray_node:
            # Update ray
            self._c_ray.setOrigin(target_center)
            # Direction is from target to camera
            direction = Vec3(x, -y, z)
            direction.normalize()
            self._c_ray.setDirection(direction)

            self._c_trav.traverse(self._base.render)

            if self._c_queue.getNumEntries() > 0:
                self._c_queue.sortEntries()
                entry = self._c_queue.getEntry(0)
                hit_pos = entry.getSurfacePoint(self._base.render)

                # Calculate distance to hit
                hit_vec = hit_pos - target_center
                hit_dist = hit_vec.length()

                # If hit is closer than desired distance, clamp
                if hit_dist < self._distance:
                    actual_distance = max(self.settings.min_distance, hit_dist - 0.5)

        # Recalculate position with actual distance
        x = actual_distance * cos(rad_pitch) * sin(-rad_yaw)
        y = actual_distance * cos(rad_pitch) * cos(-rad_yaw)
        z = actual_distance * sin(rad_pitch)

        cam_pos = target_center + Vec3(x, -y, z)

        self._base.camera.setPos(cam_pos)
        self._base.camera.lookAt(target_center)

        return task.cont

    def _setup_window(self) -> None:
        assert self._base is not None
        window = self._base.win
        if window is None:
            return

        width = window.getXSize() if hasattr(window, "getXSize") else 0
        height = window.getYSize() if hasattr(window, "getYSize") else 0
        self._window_center = (width // 2, height // 2) if width and height else None
