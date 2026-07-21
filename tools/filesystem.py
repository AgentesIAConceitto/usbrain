"""Filesystem tool."""
from __future__ import annotations

import difflib
import hashlib
import json
from pathlib import Path
import shutil
from typing import Any
import urllib.request
import zipfile

from tools.base import Tool


class ReadFileTool(Tool):
    """Read contents of a file."""

    @property
    def name(self) -> str:
        return "read_file"

    @property
    def description(self) -> str:
        return "Read the contents of a file on the local filesystem."

    @property
    def parameters(self) -> dict[str, Any]:
        return {
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Absolute or relative path to the file",
                },
                "max_lines": {
                    "type": "integer",
                    "description": "Maximum lines to read",
                    "default": 200,
                },
            },
            "required": ["path"],
        }

    async def execute(self, path: str, max_lines: int = 200) -> str:
        try:
            p = Path(path).expanduser().resolve()
            if not p.exists():
                return f"File not found: {path}"
            if not p.is_file():
                return f"Not a file: {path}"
            lines = p.read_text(encoding="utf-8", errors="replace").splitlines()
            if len(lines) > max_lines:
                content = "\n".join(lines[:max_lines])
                content += f"\n\n[... {len(lines) - max_lines} more lines truncated]"
            else:
                content = "\n".join(lines)
            return content
        except Exception as e:
            return f"Error reading file: {e}"


class WriteFileTool(Tool):
    """Write content to a file."""

    @property
    def name(self) -> str:
        return "write_file"

    @property
    def description(self) -> str:
        return "Write content to a file. Creates parent directories if needed."

    @property
    def parameters(self) -> dict[str, Any]:
        return {
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to write",
                },
                "content": {
                    "type": "string",
                    "description": "Content to write",
                },
            },
            "required": ["path", "content"],
        }

    async def execute(self, path: str, content: str) -> str:
        try:
            p = Path(path).expanduser().resolve()
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_text(content, encoding="utf-8")
            return f"✅ Written {len(content)} bytes to {path}"
        except Exception as e:
            return f"Error writing file: {e}"


class CreateDirectoryTool(Tool):
    """Create a directory."""

    @property
    def name(self) -> str:
        return "create_directory"

    @property
    def description(self) -> str:
        return "Create a directory path, including parent folders when needed."

    @property
    def parameters(self) -> dict[str, Any]:
        return {
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Directory path to create",
                },
            },
            "required": ["path"],
        }

    async def execute(self, path: str) -> str:
        try:
            p = Path(path).expanduser().resolve()
            p.mkdir(parents=True, exist_ok=True)
            return f"✅ Directory ready: {p}"
        except Exception as e:
            return f"Error creating directory: {e}"


class ListDirectoryTool(Tool):
    """List files and folders in a directory."""

    @property
    def name(self) -> str:
        return "list_directory"

    @property
    def description(self) -> str:
        return "List files and folders in a directory, including basic type information."

    @property
    def parameters(self) -> dict[str, Any]:
        return {
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Directory path to inspect",
                },
                "max_entries": {
                    "type": "integer",
                    "description": "Maximum number of entries to return",
                    "default": 100,
                },
            },
            "required": ["path"],
        }

    async def execute(self, path: str, max_entries: int = 100) -> str:
        try:
            p = Path(path).expanduser().resolve()
            if not p.exists():
                return f"Path not found: {path}"
            if not p.is_dir():
                return f"Not a directory: {path}"

            entries = sorted(p.iterdir(), key=lambda item: (not item.is_dir(), item.name.lower()))
            if not entries:
                return f"Directory is empty: {p}"

            lines = []
            for entry in entries[:max_entries]:
                marker = "[DIR]" if entry.is_dir() else "[FILE]"
                lines.append(f"{marker} {entry.name}")

            if len(entries) > max_entries:
                lines.append(f"... {len(entries) - max_entries} more entries omitted")

            return f"Contents of {p}:\n" + "\n".join(lines)
        except Exception as e:
            return f"Error listing directory: {e}"


class CopyPathTool(Tool):
    """Copy a file or directory."""

    @property
    def name(self) -> str:
        return "copy_path"

    @property
    def description(self) -> str:
        return "Copy a file or directory to another location."

    @property
    def parameters(self) -> dict[str, Any]:
        return {
            "type": "object",
            "properties": {
                "source": {
                    "type": "string",
                    "description": "Path to copy from",
                },
                "destination": {
                    "type": "string",
                    "description": "Path to copy to",
                },
                "overwrite": {
                    "type": "boolean",
                    "description": "Overwrite destination when it already exists",
                    "default": False,
                },
            },
            "required": ["source", "destination"],
        }

    async def execute(self, source: str, destination: str, overwrite: bool = False) -> str:
        try:
            src = Path(source).expanduser().resolve()
            dst = Path(destination).expanduser().resolve()
            if not src.exists():
                return f"Source not found: {source}"
            if dst.exists():
                if not overwrite:
                    return f"Destination already exists: {destination}"
                if dst.is_dir() and not dst.is_symlink():
                    shutil.rmtree(dst)
                else:
                    dst.unlink()

            dst.parent.mkdir(parents=True, exist_ok=True)
            if src.is_dir():
                shutil.copytree(src, dst)
                return f"✅ Copied directory: {src} -> {dst}"

            shutil.copy2(src, dst)
            return f"✅ Copied file: {src} -> {dst}"
        except Exception as e:
            return f"Error copying path: {e}"


class MovePathTool(Tool):
    """Move or rename a file or directory."""

    @property
    def name(self) -> str:
        return "move_path"

    @property
    def description(self) -> str:
        return "Move or rename a file or directory."

    @property
    def parameters(self) -> dict[str, Any]:
        return {
            "type": "object",
            "properties": {
                "source": {
                    "type": "string",
                    "description": "Path to move from",
                },
                "destination": {
                    "type": "string",
                    "description": "Path to move to",
                },
                "overwrite": {
                    "type": "boolean",
                    "description": "Overwrite destination when it already exists",
                    "default": False,
                },
            },
            "required": ["source", "destination"],
        }

    async def execute(self, source: str, destination: str, overwrite: bool = False) -> str:
        try:
            src = Path(source).expanduser().resolve()
            dst = Path(destination).expanduser().resolve()
            if not src.exists():
                return f"Source not found: {source}"
            if dst.exists():
                if not overwrite:
                    return f"Destination already exists: {destination}"
                if dst.is_dir() and not dst.is_symlink():
                    shutil.rmtree(dst)
                else:
                    dst.unlink()

            dst.parent.mkdir(parents=True, exist_ok=True)
            shutil.move(str(src), str(dst))
            return f"✅ Moved path: {src} -> {dst}"
        except Exception as e:
            return f"Error moving path: {e}"


class CreateZipTool(Tool):
    """Create a zip archive."""

    @property
    def name(self) -> str:
        return "create_zip"

    @property
    def description(self) -> str:
        return "Create a .zip archive from a file or directory."

    @property
    def parameters(self) -> dict[str, Any]:
        return {
            "type": "object",
            "properties": {
                "source": {
                    "type": "string",
                    "description": "File or directory to compress",
                },
                "destination_zip": {
                    "type": "string",
                    "description": "Target .zip file path",
                },
                "overwrite": {
                    "type": "boolean",
                    "description": "Overwrite the destination archive if it exists",
                    "default": False,
                },
            },
            "required": ["source", "destination_zip"],
        }

    async def execute(self, source: str, destination_zip: str, overwrite: bool = False) -> str:
        try:
            src = Path(source).expanduser().resolve()
            dst = Path(destination_zip).expanduser().resolve()
            if not src.exists():
                return f"Source not found: {source}"
            if dst.exists() and not overwrite:
                return f"Destination archive already exists: {destination_zip}"

            dst.parent.mkdir(parents=True, exist_ok=True)
            mode = "w"
            with zipfile.ZipFile(dst, mode, compression=zipfile.ZIP_DEFLATED) as archive:
                if src.is_file():
                    archive.write(src, arcname=src.name)
                else:
                    for child in src.rglob("*"):
                        if child.is_file():
                            archive.write(child, arcname=child.relative_to(src.parent))
            return f"✅ Zip created: {dst}"
        except Exception as e:
            return f"Error creating zip: {e}"


class ExtractZipTool(Tool):
    """Extract a zip archive."""

    @property
    def name(self) -> str:
        return "extract_zip"

    @property
    def description(self) -> str:
        return "Extract a .zip archive into a target directory."

    @property
    def parameters(self) -> dict[str, Any]:
        return {
            "type": "object",
            "properties": {
                "zip_path": {
                    "type": "string",
                    "description": "Path to the .zip archive",
                },
                "destination_dir": {
                    "type": "string",
                    "description": "Directory where files will be extracted",
                },
            },
            "required": ["zip_path", "destination_dir"],
        }

    async def execute(self, zip_path: str, destination_dir: str) -> str:
        try:
            src = Path(zip_path).expanduser().resolve()
            dst = Path(destination_dir).expanduser().resolve()
            if not src.exists():
                return f"Archive not found: {zip_path}"
            if not src.is_file():
                return f"Not a file: {zip_path}"

            dst.mkdir(parents=True, exist_ok=True)
            with zipfile.ZipFile(src, "r") as archive:
                archive.extractall(dst)
                file_count = len(archive.infolist())
            return f"✅ Extracted {file_count} item(s) to {dst}"
        except Exception as e:
            return f"Error extracting zip: {e}"


class DownloadUrlTool(Tool):
    """Download a file from a URL."""

    @property
    def name(self) -> str:
        return "download_url"

    @property
    def description(self) -> str:
        return "Download a file from a URL and save it to disk."

    @property
    def parameters(self) -> dict[str, Any]:
        return {
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "Source URL to download",
                },
                "destination": {
                    "type": "string",
                    "description": "Local path where the file will be saved",
                },
                "overwrite": {
                    "type": "boolean",
                    "description": "Overwrite destination if it already exists",
                    "default": False,
                },
            },
            "required": ["url", "destination"],
        }

    async def execute(self, url: str, destination: str, overwrite: bool = False) -> str:
        try:
            dst = Path(destination).expanduser().resolve()
            if dst.exists() and not overwrite:
                return f"Destination already exists: {destination}"

            dst.parent.mkdir(parents=True, exist_ok=True)
            with urllib.request.urlopen(url, timeout=30) as response:
                content = response.read()
            dst.write_bytes(content)
            return f"✅ Downloaded {len(content)} bytes to {dst}"
        except Exception as e:
            return f"Error downloading URL: {e}"


class FileChecksumTool(Tool):
    """Calculate file checksums."""

    @property
    def name(self) -> str:
        return "file_checksum"

    @property
    def description(self) -> str:
        return "Calculate a checksum or hash for a file."

    @property
    def parameters(self) -> dict[str, Any]:
        return {
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file",
                },
                "algorithm": {
                    "type": "string",
                    "enum": ["md5", "sha1", "sha256", "sha512"],
                    "description": "Hash algorithm",
                    "default": "sha256",
                },
            },
            "required": ["path"],
        }

    async def execute(self, path: str, algorithm: str = "sha256") -> str:
        try:
            p = Path(path).expanduser().resolve()
            if not p.exists():
                return f"File not found: {path}"
            if not p.is_file():
                return f"Not a file: {path}"

            hasher = hashlib.new(algorithm)
            with p.open("rb") as fh:
                while chunk := fh.read(1024 * 1024):
                    hasher.update(chunk)
            return f"{algorithm}:{hasher.hexdigest()}"
        except Exception as e:
            return f"Error calculating checksum: {e}"


class CompareFilesTool(Tool):
    """Compare two text files."""

    @property
    def name(self) -> str:
        return "compare_files"

    @property
    def description(self) -> str:
        return "Compare two text files and return a unified diff summary."

    @property
    def parameters(self) -> dict[str, Any]:
        return {
            "type": "object",
            "properties": {
                "left_path": {
                    "type": "string",
                    "description": "First file path",
                },
                "right_path": {
                    "type": "string",
                    "description": "Second file path",
                },
                "max_lines": {
                    "type": "integer",
                    "description": "Maximum diff lines to return",
                    "default": 200,
                },
            },
            "required": ["left_path", "right_path"],
        }

    async def execute(self, left_path: str, right_path: str, max_lines: int = 200) -> str:
        try:
            left = Path(left_path).expanduser().resolve()
            right = Path(right_path).expanduser().resolve()
            if not left.exists():
                return f"File not found: {left_path}"
            if not right.exists():
                return f"File not found: {right_path}"
            if not left.is_file() or not right.is_file():
                return "Both paths must be files."

            left_lines = left.read_text(encoding="utf-8", errors="replace").splitlines()
            right_lines = right.read_text(encoding="utf-8", errors="replace").splitlines()
            diff = list(
                difflib.unified_diff(
                    left_lines,
                    right_lines,
                    fromfile=str(left),
                    tofile=str(right),
                    lineterm="",
                )
            )
            if not diff:
                return "✅ Files are identical."

            visible = diff[:max_lines]
            if len(diff) > max_lines:
                visible.append(f"... {len(diff) - max_lines} more diff lines omitted")
            return "\n".join(visible)
        except Exception as e:
            return f"Error comparing files: {e}"


class ReplaceInFileTool(Tool):
    """Replace text inside a file."""

    @property
    def name(self) -> str:
        return "replace_in_file"

    @property
    def description(self) -> str:
        return "Replace text inside a file and save the result."

    @property
    def parameters(self) -> dict[str, Any]:
        return {
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "File to update",
                },
                "search": {
                    "type": "string",
                    "description": "Exact text to search for",
                },
                "replace": {
                    "type": "string",
                    "description": "Replacement text",
                },
                "count": {
                    "type": "integer",
                    "description": "Maximum number of replacements. Use 0 for all.",
                    "default": 0,
                },
            },
            "required": ["path", "search", "replace"],
        }

    async def execute(self, path: str, search: str, replace: str, count: int = 0) -> str:
        try:
            p = Path(path).expanduser().resolve()
            if not p.exists():
                return f"File not found: {path}"
            if not p.is_file():
                return f"Not a file: {path}"
            if search == "":
                return "Search text cannot be empty."

            text = p.read_text(encoding="utf-8", errors="replace")
            if search not in text:
                return "No occurrences found."

            replacements = text.count(search) if count == 0 else min(text.count(search), count)
            updated = text.replace(search, replace, count if count > 0 else -1)
            p.write_text(updated, encoding="utf-8")
            return f"✅ Replaced {replacements} occurrence(s) in {p}"
        except Exception as e:
            return f"Error replacing text in file: {e}"


class DirectoryManifestTool(Tool):
    """Generate a manifest for a directory."""

    @property
    def name(self) -> str:
        return "directory_manifest"

    @property
    def description(self) -> str:
        return "Generate a manifest of files inside a directory, including size and relative path."

    @property
    def parameters(self) -> dict[str, Any]:
        return {
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Directory to inspect",
                },
                "max_entries": {
                    "type": "integer",
                    "description": "Maximum number of files to include",
                    "default": 500,
                },
            },
            "required": ["path"],
        }

    async def execute(self, path: str, max_entries: int = 500) -> str:
        try:
            root = Path(path).expanduser().resolve()
            if not root.exists():
                return f"Path not found: {path}"
            if not root.is_dir():
                return f"Not a directory: {path}"

            files: list[dict[str, Any]] = []
            for child in sorted(root.rglob("*")):
                if not child.is_file():
                    continue
                stat = child.stat()
                files.append(
                    {
                        "path": str(child.relative_to(root)),
                        "size": stat.st_size,
                    }
                )
                if len(files) >= max_entries:
                    break

            payload = {
                "root": str(root),
                "file_count": len(files),
                "files": files,
            }
            return json.dumps(payload, ensure_ascii=True, indent=2)
        except Exception as e:
            return f"Error generating directory manifest: {e}"


class CompareDirectoriesTool(Tool):
    """Compare directory contents by relative file list."""

    @property
    def name(self) -> str:
        return "compare_directories"

    @property
    def description(self) -> str:
        return "Compare two directories by relative file paths and sizes."

    @property
    def parameters(self) -> dict[str, Any]:
        return {
            "type": "object",
            "properties": {
                "left_path": {
                    "type": "string",
                    "description": "First directory",
                },
                "right_path": {
                    "type": "string",
                    "description": "Second directory",
                },
                "max_differences": {
                    "type": "integer",
                    "description": "Maximum number of differences to return",
                    "default": 200,
                },
            },
            "required": ["left_path", "right_path"],
        }

    @staticmethod
    def _collect_directory_state(root: Path) -> dict[str, int]:
        state: dict[str, int] = {}
        for child in root.rglob("*"):
            if child.is_file():
                state[str(child.relative_to(root))] = child.stat().st_size
        return state

    async def execute(self, left_path: str, right_path: str, max_differences: int = 200) -> str:
        try:
            left = Path(left_path).expanduser().resolve()
            right = Path(right_path).expanduser().resolve()
            if not left.exists():
                return f"Directory not found: {left_path}"
            if not right.exists():
                return f"Directory not found: {right_path}"
            if not left.is_dir() or not right.is_dir():
                return "Both paths must be directories."

            left_state = self._collect_directory_state(left)
            right_state = self._collect_directory_state(right)

            left_only = sorted(set(left_state) - set(right_state))
            right_only = sorted(set(right_state) - set(left_state))
            changed = sorted(
                path for path in set(left_state).intersection(right_state)
                if left_state[path] != right_state[path]
            )

            if not left_only and not right_only and not changed:
                return "✅ Directories are identical by file list and size."

            lines: list[str] = []
            for path in left_only:
                lines.append(f"ONLY_LEFT {path}")
            for path in right_only:
                lines.append(f"ONLY_RIGHT {path}")
            for path in changed:
                lines.append(f"SIZE_DIFF {path} ({left_state[path]} != {right_state[path]})")

            visible = lines[:max_differences]
            if len(lines) > max_differences:
                visible.append(f"... {len(lines) - max_differences} more differences omitted")
            return "\n".join(visible)
        except Exception as e:
            return f"Error comparing directories: {e}"


class BatchDeleteTool(Tool):
    """Delete multiple paths in one call."""

    @property
    def name(self) -> str:
        return "batch_delete"

    @property
    def description(self) -> str:
        return "Delete multiple files or directories in a single operation."

    @property
    def parameters(self) -> dict[str, Any]:
        return {
            "type": "object",
            "properties": {
                "paths": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Paths to delete",
                },
                "recursive": {
                    "type": "boolean",
                    "description": "Delete directories recursively",
                    "default": False,
                },
            },
            "required": ["paths"],
        }

    async def execute(self, paths: list[str], recursive: bool = False) -> str:
        results: list[str] = []
        for path in paths:
            try:
                p = Path(path).expanduser().resolve()
                if not p.exists():
                    results.append(f"SKIP {path}: not found")
                    continue
                if p.is_file() or p.is_symlink():
                    p.unlink()
                    results.append(f"OK file {p}")
                elif recursive:
                    shutil.rmtree(p)
                    results.append(f"OK dir {p}")
                else:
                    p.rmdir()
                    results.append(f"OK empty dir {p}")
            except Exception as e:
                results.append(f"FAIL {path}: {e}")
        return "\n".join(results)


class DeletePathTool(Tool):
    """Delete files or directories."""

    @property
    def name(self) -> str:
        return "delete_path"

    @property
    def description(self) -> str:
        return "Delete a file or a directory. Directories can be deleted recursively."

    @property
    def parameters(self) -> dict[str, Any]:
        return {
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file or directory to delete",
                },
                "recursive": {
                    "type": "boolean",
                    "description": "When deleting a directory, remove contents recursively",
                    "default": False,
                },
            },
            "required": ["path"],
        }

    async def execute(self, path: str, recursive: bool = False) -> str:
        try:
            p = Path(path).expanduser().resolve()
            if not p.exists():
                return f"Path not found: {path}"

            if p.is_file() or p.is_symlink():
                p.unlink()
                return f"✅ Deleted file: {p}"

            if not recursive:
                p.rmdir()
                return f"✅ Deleted empty directory: {p}"

            shutil.rmtree(p)
            return f"✅ Deleted directory recursively: {p}"
        except Exception as e:
            return f"Error deleting path: {e}"
