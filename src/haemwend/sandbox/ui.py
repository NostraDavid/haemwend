"""UI overlay placeholder for the Panda3D sandbox."""

from __future__ import annotations


class SandboxUI:
    """Represent the instructional overlay shown within the sandbox."""

    def __init__(self) -> None:
        self.visible = True

    def toggle(self) -> None:
        """Switch the overlay visibility state."""
        self.visible = not self.visible

    def apply(self) -> None:
        """Render the UI overlay using Panda3D primitives."""
        raise NotImplementedError("Sandbox UI rendering will be implemented in a later phase.")
