from dataclasses import dataclass
from typing import Any


@dataclass
class Ability:
    name: str
    cooldown: float
    damage: float
    description: str = ""

    def activate(self, caster: Any, target: Any) -> None:
        """Activate the ability."""
        # Placeholder for ability logic
