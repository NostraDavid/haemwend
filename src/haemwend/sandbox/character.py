"""Character controller for the Panda3D sandbox."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

from direct.actor.Actor import Actor  # type: ignore[import-not-found]
from panda3d.core import NodePath, Vec3  # type: ignore[import-not-found]

from haemwend.sandbox.gameplay.classes import WARRIOR, CharacterClass

if TYPE_CHECKING:
    from direct.showbase.ShowBase import ShowBase  # type: ignore[import-not-found]
else:
    ShowBase = Any


class Character:
    """Represents a player character in the world."""

    def __init__(self, base: ShowBase, start_pos: Vec3 | None = None, char_class: CharacterClass = WARRIOR):
        self.base = base
        self.char_class = char_class

        # Create a root node for logic (movement, rotation)
        self.root = NodePath("character_root")
        self.root.reparentTo(base.render)

        if start_pos is None:
            start_pos = Vec3(0, 0, 0)
        self.root.setPos(start_pos)

        # Try to load a model, fallback to a simple placeholder if not found
        try:
            # Try loading Ralph (human) first
            self.actor = Actor("models/ralph", {"walk": "models/ralph-walk", "run": "models/ralph-run"})
            self.actor.setScale(0.5, 0.5, 0.5)
            # Ralph usually faces Y+, so no rotation needed
        except Exception:
            try:
                # Fallback to Panda
                self.actor = Actor("models/panda-model", {"walk": "models/panda-walk4"})
                self.actor.setScale(0.005, 0.005, 0.005)
                # Panda model faces Y-, so we rotate the geometry 180 degrees relative to root
                self.actor.setH(180)
            except Exception:
                # Fallback to a simple box
                self.actor = base.loader.loadModel("models/box")  # type: ignore[attr-defined]
                self.actor.setScale(1, 1, 2)

        self.actor.reparentTo(self.root)

        self.speed = 5.0
        self.rotation_speed = 100.0
        self.velocity = Vec3(0, 0, 0)
        self.is_moving = False
        self.is_jumping = False
        self.vertical_velocity = 0.0
        self.gravity = -9.8
        self.jump_force = 5.0

    def set_pos(self, pos: Vec3):
        self.root.setPos(pos)

    def get_pos(self) -> Vec3:
        return self.root.getPos()

    def set_h(self, h: float):
        self.root.setH(h)

    def get_h(self) -> float:
        return self.root.getH()

    def jump(self):
        if not self.is_jumping:
            self.is_jumping = True
            self.vertical_velocity = self.jump_force

    def move(self, direction: Vec3, dt: float):
        """Move the character in the given direction relative to its facing."""
        # Apply gravity
        self.vertical_velocity += self.gravity * dt

        current_pos = self.root.getPos()
        new_z = current_pos.z + self.vertical_velocity * dt

        # Simple ground collision
        if new_z <= 0:
            new_z = 0
            self.vertical_velocity = 0
            self.is_jumping = False

        self.root.setZ(new_z)

        if direction.lengthSquared() > 0:
            self.is_moving = True
            # Transform direction to be relative to character's rotation
            rotation = self.root.getQuat()
            move_vec = rotation.xform(direction)

            self.root.setPos(self.root.getPos() + move_vec * self.speed * dt)

            # Play animation if available and not playing
            if isinstance(self.actor, Actor) and self.actor.getCurrentAnim() != "walk" and not self.is_jumping:
                self.actor.loop("walk")
        else:
            self.is_moving = False
            if isinstance(self.actor, Actor) and self.actor.getCurrentAnim() == "walk" and not self.is_jumping:
                self.actor.stop()

    def cleanup(self):
        if self.actor:
            self.actor.cleanup()
            self.actor.removeNode()
        if self.root:
            self.root.removeNode()
