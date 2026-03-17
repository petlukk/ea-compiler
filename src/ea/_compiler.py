"""Subprocess wrapper around the ea binary."""

import subprocess
import platform
from pathlib import Path


_BIN_DIR = Path(__file__).parent / "bin"


def _ea_binary() -> Path:
    """Return path to bundled ea binary."""
    import sys
    name = "ea.exe" if sys.platform == "win32" else "ea"
    path = _BIN_DIR / name
    if not path.exists():
        raise RuntimeError(
            f"ea binary not found at {path}. "
            "Reinstall with: pip install --force-reinstall ea-compiler"
        )
    return path


def compiler_version() -> str:
    """Return the version string of the bundled ea binary."""
    result = subprocess.run(
        [str(_ea_binary()), "--version"],
        capture_output=True, text=True, timeout=10,
    )
    return result.stdout.strip().removeprefix("ea ")


def compile(path, *, emit_asm=False, emit_llvm=False, target="native",
            opt_level=3, avx512=False, lib=True):
    """Compile an .ea file. Returns path to output artifact."""
    raise NotImplementedError


def _resolve_target() -> str:
    """Resolve 'native' to a concrete CPU name for cache keying."""
    raise NotImplementedError
