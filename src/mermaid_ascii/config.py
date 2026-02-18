"""Centralized configuration for mermaid-ascii."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass
class RenderConfig:
    """Configuration for the rendering pipeline."""

    unicode: bool = True
    padding: int = 1
    direction_override: str | None = None
