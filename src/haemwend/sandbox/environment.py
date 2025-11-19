"""Scene composition helpers for the Panda3D sandbox."""

from __future__ import annotations

from math import sqrt
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

from haemwend.infrastructure.config_loader import (
    DEFAULT_PRIMITIVES,
    EnvironmentConfig,
    PrimitiveConfig,
    SandboxConfig,
)
from haemwend.infrastructure.logging import get_logger

__all__ = ["build_environment", "clamp_within_boundary"]


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
        node = _instantiate_primitive(
            base,
            primitive,
            logger,
            boundary_radius=environment.boundary_radius,
        )
        nodes.append(node)

    return nodes


def _instantiate_primitive(
    base: ShowBase,
    primitive: PrimitiveConfig,
    logger,
    *,
    boundary_radius: float,
) -> NodePath:
    maker = CardMaker(primitive.name)
    maker.setFrame(-0.5, 0.5, -0.5, 0.5)
    node = NodePath(maker.generate())
    node.setTwoSided(True)

    clamped_position = clamp_within_boundary(primitive.position, boundary_radius=boundary_radius)
    node.setPos(LPoint3f(*clamped_position))
    node.setScale(LVector3f(*primitive.scale))
    node.setColor(Vec4(*primitive.color))

    primitive_type = primitive.type.lower()
    if primitive_type == "cube":
        node.setHpr(45, 0, 0)
    elif primitive_type == "cylinder":
        scale = node.getScale()
        scale.componentwiseMult(LVector3f(0.6, 0.6, 1.0))
        node.setScale(scale)
    elif primitive_type == "sphere":
        scale = node.getScale()
        scale.componentwiseMult(LVector3f(0.8, 0.8, 0.8))
        node.setScale(scale)

    node.reparentTo(base.render)
    logger.info(
        "sandbox.environment.primitive",
        name=primitive.name,
        type=primitive.type,
        position=list(clamped_position),
        original_position=list(primitive.position),
        clamped=clamped_position != primitive.position,
    )
    return node


def _default_primitives() -> list[PrimitiveConfig]:
    return list(DEFAULT_PRIMITIVES)


def clamp_within_boundary(
    position: tuple[float, float, float],
    *,
    boundary_radius: float,
) -> tuple[float, float, float]:
    if boundary_radius <= 0:
        return position

    x, y, z = position
    planar_distance = sqrt(x * x + y * y)
    if planar_distance <= boundary_radius:
        return position

    scale = boundary_radius / planar_distance
    clamped = (x * scale, y * scale, z)
    return clamped
