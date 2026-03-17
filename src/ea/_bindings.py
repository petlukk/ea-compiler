"""Runtime ctypes binding generation from .ea.json metadata."""

import json
from pathlib import Path


class CompileError(RuntimeError):
    """Raised when ea compilation fails."""

    def __init__(self, message: str, stderr: str, exit_code: int):
        super().__init__(message)
        self.stderr = stderr
        self.exit_code = exit_code


def _parse_metadata(json_path: Path) -> list:
    """Parse .ea.json metadata file. Returns list of export dicts."""
    with open(json_path) as f:
        data = json.load(f)
    return data.get("exports", [])
