"""Heads-up display helpers for the Panda3D sandbox."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

from direct.gui.DirectGui import DirectFrame  # type: ignore[import-not-found]
from direct.gui.OnscreenText import OnscreenText  # type: ignore[import-not-found]
from panda3d.core import TextNode  # type: ignore[import-not-found]

if TYPE_CHECKING:
    from direct.showbase.ShowBase import ShowBase  # type: ignore[import-not-found]
else:  # pragma: no cover - runtime fallback when Panda3D is unavailable
    ShowBase = Any  # type: ignore[misc,assignment]


class SandboxUI:
    """Render and control the instructional overlay shown within the sandbox."""

    _TITLE = "Haemwend Sandbox"

    def __init__(self, *, help_lines: list[str] | tuple[str, ...] | None = None) -> None:
        self.visible = True
        self._base: ShowBase | None = None
        self._panel: DirectFrame | None = None
        self._title_label: OnscreenText | None = None
        self._instruction_labels: list[OnscreenText] = []
        self._boundary_label: Any | None = None
        self._body_lines: list[str] = list(help_lines) if help_lines is not None else list(self._default_lines())

    # Lifecycle ------------------------------------------------------

    def bind(self, base: ShowBase) -> None:
        """Attach the UI overlay to the provided Panda3D application."""

        if self._base is base:
            if not self.visible:
                self.show()
            return

        self.unbind()
        self._base = base
        self._create_overlay()
        self.show()

    def unbind(self) -> None:
        """Detach overlay nodes from the current Panda3D application."""

        if self._panel is not None:
            self._panel.destroy()
        if self._title_label is not None:
            self._title_label.removeNode()
        for label in self._instruction_labels:
            label.removeNode()
        boundary = self._boundary_label
        if boundary is not None:
            boundary.removeNode()

        self._panel = None
        self._title_label = None
        self._instruction_labels.clear()
        self._boundary_label = None
        self._base = None

    # Visibility -----------------------------------------------------

    def toggle(self) -> None:
        """Switch the overlay visibility state."""

        if self.visible:
            self.hide()
        else:
            self.show()

    def show(self) -> None:
        """Display the overlay if it has been created."""

        if self._panel is not None:
            self._panel.show()
        if self._title_label is not None:
            self._title_label.show()
        for label in self._instruction_labels:
            label.show()
        boundary = self._boundary_label
        if boundary is not None:
            boundary.show()
        self.visible = True

    def hide(self) -> None:
        """Hide the overlay if it has been created."""

        if self._panel is not None:
            self._panel.hide()
        if self._title_label is not None:
            self._title_label.hide()
        for label in self._instruction_labels:
            label.hide()
        boundary = self._boundary_label
        if boundary is not None:
            boundary.hide()
        self.visible = False

    # Rendering ------------------------------------------------------

    def _create_overlay(self) -> None:
        if self._base is None:
            return

        aspect2d = getattr(self._base, "aspect2d", None)
        if aspect2d is None:
            return

        self._panel = DirectFrame(
            parent=aspect2d,
            frameColor=(0.05, 0.05, 0.08, 0.75),
            frameSize=(-1.05, -0.1, -0.55, 0.3),
            pos=(1.1, 0, 0.72),
        )

        self._instruction_labels.clear()

        title = OnscreenText(
            text=self._TITLE,
            parent=self._panel,
            pos=(-0.93, 0.2),
            fg=(1.0, 0.96, 0.85, 1.0),
            align=TextNode.ALeft,
            scale=0.06,
            mayChange=False,
        )
        self._title_label = title

        line_height = 0.1
        start_y = 0.05
        for index, body_text in enumerate(self._body_lines):
            label = OnscreenText(
                text=body_text,
                parent=self._panel,
                pos=(-0.93, start_y - index * line_height),
                fg=(0.85, 0.9, 1.0, 1.0),
                align=TextNode.ALeft,
                scale=0.045,
                mayChange=True,
            )
            self._instruction_labels.append(label)

        boundary = OnscreenText(
            text="",
            parent=self._panel,
            pos=(-0.93, -0.45),
            fg=(1.0, 0.7, 0.4, 1.0),
            align=TextNode.ALeft,
            scale=0.043,
            mayChange=True,
        )
        boundary.hide()
        self._boundary_label = boundary

    # Feedback -------------------------------------------------------

    def update_boundary_feedback(self, ratio: float) -> None:
        """Display a hint when the camera approaches or exits the boundary."""

    # Content --------------------------------------------------------

    def set_control_instructions(self, lines: list[str] | tuple[str, ...]) -> None:
        """Update the instruction text displayed in the overlay."""

        new_lines = [line.strip() for line in lines if line.strip()]
        if not new_lines:
            new_lines = list(self._default_lines())
        self._body_lines = new_lines

        base = self._base
        if base is None:
            return

        was_visible = self.visible
        # Rebuild overlay to ensure label count matches new lines.
        self.unbind()
        self._base = base
        self._create_overlay()
        if was_visible:
            self.show()
        else:
            self.hide()

    @staticmethod
    def _default_lines() -> tuple[str, ...]:
        return (
            "W / A / S / D — walk",
            "Mouse — look",
            "Shift — sprint",
            "Space / Ctrl — rise & descend",
            "H — toggle help",
            "Esc — quit window",
        )
