from dataclasses import dataclass, field

from .abilities import Ability


@dataclass
class CharacterClass:
    name: str
    base_health: float
    base_mana: float
    abilities: list[Ability] = field(default_factory=list)


# Example Classes
WARRIOR = CharacterClass(name="Warrior", base_health=100.0, base_mana=0.0, abilities=[])

MAGE = CharacterClass(name="Mage", base_health=60.0, base_mana=100.0, abilities=[])
