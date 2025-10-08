"""Load and validate sandbox configuration from TOML files."""

from __future__ import annotations

import tomllib
from collections.abc import Iterable, Mapping, Sequence
from dataclasses import dataclass, field, replace
from pathlib import Path
from typing import Any, cast

__all__ = [
    "DEFAULT_CAMERA_CONFIG",
    "DEFAULT_CONFIG_PATH",
    "DEFAULT_ENVIRONMENT_CONFIG",
    "DEFAULT_PRIMITIVES",
    "DEFAULT_SANDBOX_CONFIG",
    "CameraConfig",
    "EnvironmentConfig",
    "PrimitiveConfig",
    "SandboxConfig",
    "SandboxConfigError",
    "default_sandbox_config",
    "load_sandbox_config",
    "parse_sandbox_config",
]


class SandboxConfigError(ValueError):
    """Raised when the sandbox configuration contains invalid data."""


@dataclass(slots=True)
class PrimitiveConfig:
    """Configuration for a single primitive mesh in the sandbox scene."""

    name: str
    type: str
    position: tuple[float, float, float]
    scale: tuple[float, float, float]
    color: tuple[float, float, float, float]


@dataclass(slots=True)
class EnvironmentConfig:
    """Sandbox environment layout and constraints."""

    boundary_radius: float = 30.0
    primitives: tuple[PrimitiveConfig, ...] = field(default_factory=tuple)


@dataclass(slots=True)
class CameraConfig:
    """Camera tuning options for sandbox traversal."""

    move_speed: float = 5.5
    sprint_multiplier: float = 1.8
    mouse_sensitivity: float = 0.15
    vertical_look_limit: float = 82.0


@dataclass(slots=True)
class SandboxConfig:
    """Top-level configuration for the Panda3D sandbox feature."""

    enabled: bool = True
    camera: CameraConfig = field(default_factory=CameraConfig)
    environment: EnvironmentConfig = field(default_factory=EnvironmentConfig)


DEFAULT_CONFIG_PATH: Path = Path(__file__).resolve().parents[3] / "config" / "sandbox.toml"

_MIN_BOUNDARY_RADIUS = 5.0
_MAX_BOUNDARY_RADIUS = 150.0
_MIN_MOVE_SPEED = 0.5
_MAX_MOVE_SPEED = 30.0
_MIN_SPRINT_MULTIPLIER = 1.0
_MAX_SPRINT_MULTIPLIER = 6.0
_MIN_MOUSE_SENSITIVITY = 0.01
_MAX_MOUSE_SENSITIVITY = 1.0
_MIN_VERTICAL_LOOK_LIMIT = 30.0
_MAX_VERTICAL_LOOK_LIMIT = 90.0
_MIN_SCALE = 0.1
_MAX_SCALE = 20.0
_ALLOWED_PRIMITIVE_TYPES = {"cube", "cylinder", "sphere", "card", "plane"}


DEFAULT_PRIMITIVES: tuple[PrimitiveConfig, ...] = (
    PrimitiveConfig(
        name="central_obelisk",
        type="cylinder",
        position=(0.0, 0.0, 0.0),
        scale=(1.0, 1.0, 6.0),
        color=(0.6, 0.5, 0.8, 1.0),
    ),
    PrimitiveConfig(
        name="west_pillar",
        type="cube",
        position=(-8.0, 5.0, 0.0),
        scale=(2.0, 2.0, 4.0),
        color=(0.3, 0.7, 0.5, 1.0),
    ),
    PrimitiveConfig(
        name="north_orb",
        type="sphere",
        position=(4.0, 12.0, 2.5),
        scale=(1.5, 1.5, 1.5),
        color=(0.9, 0.6, 0.2, 1.0),
    ),
)

DEFAULT_CAMERA_CONFIG = CameraConfig()
DEFAULT_ENVIRONMENT_CONFIG = EnvironmentConfig(boundary_radius=30.0, primitives=DEFAULT_PRIMITIVES)
DEFAULT_SANDBOX_CONFIG = SandboxConfig(
    enabled=True,
    camera=DEFAULT_CAMERA_CONFIG,
    environment=DEFAULT_ENVIRONMENT_CONFIG,
)


def load_sandbox_config(path: Path | None = None) -> SandboxConfig:
    """Load sandbox configuration from disk and validate it."""

    config_path = path or DEFAULT_CONFIG_PATH
    if not config_path.exists():
        return SandboxConfig()

    try:
        with config_path.open("rb") as handle:
            data = tomllib.load(handle)
    except OSError as exc:  # pragma: no cover - filesystem failure
        raise SandboxConfigError(_message("unable to read sandbox configuration", config_path)) from exc

    return parse_sandbox_config(data, source=config_path)


def parse_sandbox_config(payload: Mapping[str, Any], *, source: Path | None = None) -> SandboxConfig:
    """Convert a raw TOML payload into validated sandbox dataclasses."""

    enabled = bool(payload.get("enabled", True))

    camera_section = _as_dict(payload.get("camera"), "camera", source)
    camera = _parse_camera(camera_section, source)

    environment_section = _as_dict(payload.get("environment"), "environment", source)
    environment = _parse_environment(environment_section, source)

    return SandboxConfig(enabled=enabled, camera=camera, environment=environment)


def _parse_camera(section: Mapping[str, Any], source: Path | None) -> CameraConfig:
    move_speed = _bounded_float(
        section.get("move_speed", DEFAULT_CAMERA_CONFIG.move_speed),
        "camera.move_speed",
        source,
        min_value=_MIN_MOVE_SPEED,
        max_value=_MAX_MOVE_SPEED,
    )
    sprint_multiplier = _bounded_float(
        section.get("sprint_multiplier", DEFAULT_CAMERA_CONFIG.sprint_multiplier),
        "camera.sprint_multiplier",
        source,
        min_value=_MIN_SPRINT_MULTIPLIER,
        max_value=_MAX_SPRINT_MULTIPLIER,
    )
    mouse_sensitivity = _bounded_float(
        section.get("mouse_sensitivity", DEFAULT_CAMERA_CONFIG.mouse_sensitivity),
        "camera.mouse_sensitivity",
        source,
        min_value=_MIN_MOUSE_SENSITIVITY,
        max_value=_MAX_MOUSE_SENSITIVITY,
    )
    vertical_look_limit = _bounded_float(
        section.get("vertical_look_limit", DEFAULT_CAMERA_CONFIG.vertical_look_limit),
        "camera.vertical_look_limit",
        source,
        min_value=_MIN_VERTICAL_LOOK_LIMIT,
        max_value=_MAX_VERTICAL_LOOK_LIMIT,
    )

    return CameraConfig(
        move_speed=move_speed,
        sprint_multiplier=sprint_multiplier,
        mouse_sensitivity=mouse_sensitivity,
        vertical_look_limit=vertical_look_limit,
    )


def _parse_environment(section: Mapping[str, Any], source: Path | None) -> EnvironmentConfig:
    boundary_radius = _bounded_float(
        section.get("boundary_radius", DEFAULT_ENVIRONMENT_CONFIG.boundary_radius),
        "environment.boundary_radius",
        source,
        min_value=_MIN_BOUNDARY_RADIUS,
        max_value=_MAX_BOUNDARY_RADIUS,
    )

    primitives_payload = section.get("primitives", [])
    primitives_data = _as_sequence(primitives_payload, "environment.primitives", source)

    primitives: list[PrimitiveConfig] = []
    for idx, raw in enumerate(primitives_data):
        primitive_section = _as_dict(raw, f"environment.primitives[{idx}]", source)
        name = _as_str(primitive_section.get("name"), f"environment.primitives[{idx}].name", source)
        mesh_type = _primitive_type(
            primitive_section.get("type"),
            field=f"environment.primitives[{idx}].type",
            source=source,
        )
        position = _vector3(
            primitive_section.get("position", (0.0, 0.0, 0.0)),
            field=f"environment.primitives[{idx}].position",
            source=source,
        )
        scale = _vector3(
            primitive_section.get("scale", (1.0, 1.0, 1.0)),
            field=f"environment.primitives[{idx}].scale",
            source=source,
        )
        _validate_scale(scale, f"environment.primitives[{idx}].scale", source)
        color = _vector4(
            primitive_section.get("color", (1.0, 1.0, 1.0, 1.0)),
            field=f"environment.primitives[{idx}].color",
            source=source,
        )
        if any(component < 0 or component > 1 for component in color):
            raise SandboxConfigError(
                _message(
                    f"environment.primitives[{idx}].color components must be between 0 and 1",
                    source,
                )
            )
        primitives.append(
            PrimitiveConfig(
                name=name,
                type=mesh_type,
                position=position,
                scale=scale,
                color=color,
            )
        )

    if not primitives:
        primitives = list(DEFAULT_PRIMITIVES)

    return EnvironmentConfig(boundary_radius=boundary_radius, primitives=tuple(primitives))


def _positive_float(value: Any, field: str, source: Path | None) -> float:
    number = _coerce_float(value, field=field, source=source)
    if number <= 0:
        raise SandboxConfigError(_message(f"{field} must be greater than zero", source))
    return number


def _bounded_float(
    value: Any,
    field: str,
    source: Path | None,
    *,
    min_value: float,
    max_value: float | None,
) -> float:
    number = _coerce_float(value, field=field, source=source)
    if number < min_value or (max_value is not None and number > max_value):
        if max_value is None:
            raise SandboxConfigError(_message(f"{field} must be >= {min_value}", source))
        raise SandboxConfigError(_message(f"{field} must be between {min_value} and {max_value}", source))
    return number


def _vector3(value: Any, *, field: str, source: Path | None) -> tuple[float, float, float]:
    sequence = _as_sequence(value, field, source)
    if len(sequence) != 3:
        raise SandboxConfigError(_message(f"{field} must contain exactly 3 elements", source))
    converted = tuple(_coerce_float(element, field=field, source=source) for element in sequence)
    return cast("tuple[float, float, float]", converted)


def _vector4(value: Any, *, field: str, source: Path | None) -> tuple[float, float, float, float]:
    sequence = _as_sequence(value, field, source)
    if len(sequence) != 4:
        raise SandboxConfigError(_message(f"{field} must contain exactly 4 elements", source))
    converted = tuple(_coerce_float(element, field=field, source=source) for element in sequence)
    return cast("tuple[float, float, float, float]", converted)


def _validate_scale(scale: Iterable[float], field: str, source: Path | None) -> None:
    for component in scale:
        if component < _MIN_SCALE or component > _MAX_SCALE:
            raise SandboxConfigError(
                _message(f"{field} components must be between {_MIN_SCALE} and {_MAX_SCALE}", source)
            )


def _primitive_type(value: Any, *, field: str, source: Path | None) -> str:
    primitive_type = _as_str(value, field, source).lower()
    if primitive_type not in _ALLOWED_PRIMITIVE_TYPES:
        allowed = ", ".join(sorted(_ALLOWED_PRIMITIVE_TYPES))
        raise SandboxConfigError(_message(f"{field} must be one of: {allowed}", source))
    return primitive_type


def _coerce_float(value: Any, *, field: str, source: Path | None) -> float:
    if isinstance(value, (int, float)):
        return float(value)
    if isinstance(value, str):
        try:
            return float(value)
        except ValueError as exc:
            raise SandboxConfigError(_message(f"{field} must be a number", source)) from exc
    raise SandboxConfigError(_message(f"{field} must be a number", source))


def _as_sequence(value: Any, field: str, source: Path | None) -> Sequence[Any]:
    if isinstance(value, Sequence) and not isinstance(value, (str, bytes, bytearray)):
        return value
    raise SandboxConfigError(_message(f"{field} must be a sequence", source))


def _as_dict(value: Any, field: str, source: Path | None) -> dict[str, Any]:
    if value is None:
        return {}
    if isinstance(value, Mapping):
        return {str(key): val for key, val in value.items()}
    raise SandboxConfigError(_message(f"{field} must be a table", source))


def _as_str(value: Any, field: str, source: Path | None) -> str:
    if isinstance(value, str) and value.strip():
        return value
    raise SandboxConfigError(_message(f"{field} must be a non-empty string", source))


def _message(message: str, source: Path | None) -> str:
    if source is None:
        return message
    return f"{message} (file: {source})"


def default_sandbox_config() -> SandboxConfig:
    """Return a deep copy of the baseline sandbox configuration."""

    return SandboxConfig(
        enabled=DEFAULT_SANDBOX_CONFIG.enabled,
        camera=replace(DEFAULT_CAMERA_CONFIG),
        environment=EnvironmentConfig(
            boundary_radius=DEFAULT_ENVIRONMENT_CONFIG.boundary_radius,
            primitives=tuple(replace(primitive) for primitive in DEFAULT_PRIMITIVES),
        ),
    )
