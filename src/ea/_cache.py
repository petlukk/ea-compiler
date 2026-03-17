"""Cache management for ea.load()."""

import shutil
import sys
from pathlib import Path

CACHE_DIR_NAME = "__eacache__"


def _cache_dir(ea_file: Path, cpu: str, version: str) -> Path:
    return ea_file.parent / CACHE_DIR_NAME / f"{cpu}-{version}"


def _lib_ext() -> str:
    return ".dll" if sys.platform == "win32" else ".so"


def _is_cached(ea_file: Path, cache: Path) -> bool:
    so_file = cache / f"{ea_file.stem}{_lib_ext()}"
    if not so_file.exists():
        return False
    return so_file.stat().st_mtime > ea_file.stat().st_mtime


def clear_cache(path=None):
    """Clear cached compilations."""
    if path is None:
        target = Path.cwd() / CACHE_DIR_NAME
    else:
        p = Path(path)
        if p.is_file():
            target = p.parent / CACHE_DIR_NAME
        else:
            target = p / CACHE_DIR_NAME
    if target.is_dir():
        shutil.rmtree(target)


def load(path, *, target="native", opt_level=3, avx512=False):
    """Compile, cache, and load an .ea kernel. Returns a module-like object."""
    raise NotImplementedError
