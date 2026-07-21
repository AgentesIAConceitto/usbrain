"""Tool registry — adapted from Nanobot."""
from __future__ import annotations

from typing import Any

from tools.base import Tool


class ToolRegistry:
    """Registry for agent tools. Allows dynamic registration and execution."""

    def __init__(self):
        self._tools: dict[str, Tool] = {}

    def register(self, tool: Tool) -> None:
        """Register a tool."""
        self._tools[tool.name] = tool

    def unregister(self, name: str) -> None:
        """Unregister a tool by name."""
        self._tools.pop(name, None)

    def get(self, name: str) -> Tool | None:
        """Get a tool by name."""
        return self._tools.get(name)

    def list_tools(self) -> list[Tool]:
        """List all registered tools."""
        return list(self._tools.values())

    def to_openai_schemas(self) -> list[dict[str, Any]]:
        """Get all tool schemas in OpenAI format."""
        return [t.to_openai_schema() for t in self._tools.values()]

    async def execute(self, name: str, **kwargs: Any) -> str:
        """Execute a tool by name with kwargs."""
        tool = self.get(name)
        if not tool:
            return f"Error: tool '{name}' not found"
        try:
            result = await tool.execute(**kwargs)
            return str(result)
        except Exception as e:
            return f"Error executing '{name}': {e}"
