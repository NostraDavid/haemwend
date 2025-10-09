"""Logging helpers backed by structlog for the sandbox feature."""

from __future__ import annotations

import logging
import sys
from typing import Any

import structlog
from structlog.stdlib import BoundLogger, LoggerFactory, add_logger_name

__all__ = ["configure_logging", "get_logger"]

_LOGGING_CONFIGURED = False


def configure_logging(level: int | str = "INFO") -> None:
    """Configure structlog for JSON output on stdout."""

    global _LOGGING_CONFIGURED
    if _LOGGING_CONFIGURED:
        return

    resolved_level = _resolve_level(level)
    logging.basicConfig(
        level=resolved_level,
        stream=sys.stdout,
        format="%(message)s",
        force=True,
    )

    structlog.configure(
        processors=[
            structlog.contextvars.merge_contextvars,
            structlog.processors.TimeStamper(fmt="iso", key="timestamp"),
            structlog.processors.add_log_level,
            add_logger_name,
            structlog.processors.StackInfoRenderer(),
            structlog.processors.format_exc_info,
            structlog.processors.JSONRenderer(),
        ],
        wrapper_class=structlog.make_filtering_bound_logger(resolved_level),
        logger_factory=LoggerFactory(),
        cache_logger_on_first_use=True,
    )

    _LOGGING_CONFIGURED = True


def get_logger(*, component: str | None = None, **initial_values: Any) -> BoundLogger:
    """Return a structured logger with optional component binding."""

    configure_logging()

    logger = structlog.get_logger("haemwend.sandbox")
    if component is not None:
        initial_values = {"component": component, **initial_values}
    if initial_values:
        logger = logger.bind(**initial_values)
    return logger


def _resolve_level(level: int | str) -> int:
    if isinstance(level, int):
        return level

    resolved = logging.getLevelName(level.upper())
    if isinstance(resolved, int):
        return resolved
    return logging.INFO
