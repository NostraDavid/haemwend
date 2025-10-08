"""Load and validate sandbox configuration from TOML files."""

from __future__ import annotations

import tomllib
from collections.abc import Mapping, Sequence
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, cast

__all__ = [
    "DEFAULT_CONFIG_PATH",
    "CameraConfig",
    "EnvironmentConfig",
    "PrimitiveConfig",
    "SandboxConfig",
    "SandboxConfigError",
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
    move_speed = _positive_float(section.get("move_speed", 5.5), "camera.move_speed", source)
    sprint_multiplier = _positive_float(section.get("sprint_multiplier", 1.8), "camera.sprint_multiplier", source)
    if sprint_multiplier < 1:
        raise SandboxConfigError(_message("camera.sprint_multiplier must be >= 1", source))

    mouse_sensitivity = _positive_float(section.get("mouse_sensitivity", 0.15), "camera.mouse_sensitivity", source)
    vertical_look_limit = _positive_float(
        section.get("vertical_look_limit", 82.0), "camera.vertical_look_limit", source
    )
    if not 0 < vertical_look_limit <= 90:
        raise SandboxConfigError(_message("camera.vertical_look_limit must be between 0 and 90", source))

    return CameraConfig(
        move_speed=move_speed,
        sprint_multiplier=sprint_multiplier,
        mouse_sensitivity=mouse_sensitivity,
        vertical_look_limit=vertical_look_limit,
    )


def _parse_environment(section: Mapping[str, Any], source: Path | None) -> EnvironmentConfig:
    boundary_radius = _positive_float(section.get("boundary_radius", 30.0), "environment.boundary_radius", source)
    if boundary_radius <= 0:
        raise SandboxConfigError(_message("environment.boundary_radius must be greater than zero", source))

    primitives_payload = section.get("primitives", [])
    primitives_data = _as_sequence(primitives_payload, "environment.primitives", source)

    primitives: list[PrimitiveConfig] = []
    for idx, raw in enumerate(primitives_data):
        primitive_section = _as_dict(raw, f"environment.primitives[{idx}]", source)
        name = _as_str(primitive_section.get("name"), f"environment.primitives[{idx}].name", source)
        mesh_type = _as_str(primitive_section.get("type"), f"environment.primitives[{idx}].type", source)
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

    return EnvironmentConfig(boundary_radius=boundary_radius, primitives=tuple(primitives))


def _positive_float(value: Any, field: str, source: Path | None) -> float:
    number = _coerce_float(value, field=field, source=source)
    if number <= 0:
        raise SandboxConfigError(_message(f"{field} must be greater than zero", source))
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
