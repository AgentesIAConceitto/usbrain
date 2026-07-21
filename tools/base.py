"""Base classes for agent tools — adapted from Nanobot."""
from __future__ import annotations

import typing
from abc import ABC, abstractmethod
from copy import deepcopy
from typing import Any

if typing.TYPE_CHECKING:
    pass


class Tool(ABC):
    """Agent capability: run commands, read files, search web, etc."""

    @property
    @abstractmethod
    def name(self) -> str:
        """Tool name used in function calls."""
        ...

    @property
    @abstractmethod
    def description(self) -> str:
        """Description of what the tool does."""
        ...

    @property
    @abstractmethod
    def parameters(self) -> dict[str, Any]:
        """JSON Schema for tool parameters."""
        ...

    async def execute(self, **kwargs: Any) -> str:
        """Run the tool; returns a string."""
        ...

    def to_openai_schema(self) -> dict[str, Any]:
        """OpenAI-compatible function schema."""
        return {
            "type": "function",
            "function": {
                "name": self.name,
                "description": self.description,
                "parameters": self.parameters,
            },
        }
