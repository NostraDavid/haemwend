"""Scene composition helpers for the Panda3D sandbox."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from direct.showbase.ShowBase import ShowBase  # type: ignore[import-not-found]
else:  # pragma: no cover - runtime fallback when type hints are unavailable
    ShowBase = Any  # type: ignore[misc,assignment]
from panda3d.core import (  # type: ignore[import-not-found]
    AmbientLight,
    CardMaker,
    DirectionalLight,
    LPoint3f,
    LVector3f,
    NodePath,
    Vec4,
)

from haemwend.infrastructure.config_loader import EnvironmentConfig, PrimitiveConfig, SandboxConfig
from haemwend.infrastructure.logging import get_logger

__all__ = ["build_environment"]


def build_environment(base: ShowBase, *, config: SandboxConfig) -> list[NodePath]:
    """Populate the sandbox scene with terrain, lighting, and primitives."""

    logger = get_logger(component="environment")
    nodes: list[NodePath] = []

    ground = _build_ground_plane(base)
    nodes.append(ground)
    _configure_lighting(base)

    primitive_nodes = _build_primitives(base, config.environment, logger)
    nodes.extend(primitive_nodes)

    logger.info(
        "sandbox.environment.ready",
        primitive_count=len(primitive_nodes),
        ground_node=ground.getName(),
    )
    return nodes


def _build_ground_plane(base: ShowBase) -> NodePath:
    card = CardMaker("ground")
    card.setFrame(-50, 50, -50, 50)
    ground = NodePath(card.generate())
    ground.setHpr(0, -90, 0)
    ground.setPos(0, 0, 0)
    ground.setColor(Vec4(0.18, 0.24, 0.18, 1.0))
    ground.reparentTo(base.render)
    return ground


def _configure_lighting(base: ShowBase) -> None:
    ambient = AmbientLight("ambient-light")
    ambient.setColor(Vec4(0.35, 0.35, 0.4, 1.0))
    ambient_np = base.render.attachNewNode(ambient)
    base.render.setLight(ambient_np)

    sun = DirectionalLight("sun-light")
    sun.setColor(Vec4(0.9, 0.85, 0.8, 1.0))
    sun_np = base.render.attachNewNode(sun)
    sun_np.setHpr(-35, -60, 0)
    base.render.setLight(sun_np)


def _build_primitives(
    base: ShowBase,
    environment: EnvironmentConfig,
    logger,
) -> list[NodePath]:
    nodes: list[NodePath] = []
    primitives = list(environment.primitives) if environment.primitives else _default_primitives()

    for primitive in primitives:
        node = _instantiate_primitive(base, primitive, logger)
        nodes.append(node)

    return nodes


def _instantiate_primitive(base: ShowBase, primitive: PrimitiveConfig, logger) -> NodePath:
    maker = CardMaker(primitive.name)
    maker.setFrame(-0.5, 0.5, -0.5, 0.5)
    node = NodePath(maker.generate())
    node.setTwoSided(True)

    node.setPos(LPoint3f(*primitive.position))
    node.setScale(LVector3f(*primitive.scale))
    node.setColor(Vec4(*primitive.color))

    primitive_type = primitive.type.lower()
    if primitive_type == "cube":
        node.setHpr(45, 0, 0)
    elif primitive_type == "cylinder":
        node.setScale(node.getScale() * LVector3f(0.6, 0.6, 1.0))
    elif primitive_type == "sphere":
        node.setScale(node.getScale() * LVector3f(0.8, 0.8, 0.8))

    node.reparentTo(base.render)
    logger.info(
        "sandbox.environment.primitive",
        name=primitive.name,
        type=primitive.type,
        position=list(primitive.position),
    )
    return node


def _default_primitives() -> list[PrimitiveConfig]:
    return [
        PrimitiveConfig(
            name="obelisk",
            type="cube",
            position=(0.0, 10.0, 1.5),
            scale=(1.5, 1.5, 3.0),
            color=(0.6, 0.55, 0.8, 1.0),
        ),
        PrimitiveConfig(
            name="spire",
            type="cylinder",
            position=(-8.0, -6.0, 2.0),
            scale=(1.2, 1.2, 4.0),
            color=(0.3, 0.7, 0.5, 1.0),
        ),
        PrimitiveConfig(
            name="orb",
            type="sphere",
            position=(6.0, 4.0, 2.5),
            scale=(1.5, 1.5, 1.5),
            color=(0.9, 0.6, 0.2, 1.0),
        ),
    ]
