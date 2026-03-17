"""Cache management for ea.load()."""

import shutil
import subprocess
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
    from ea._compiler import _ea_binary, _resolve_target, compiler_version
    from ea._bindings import CompileError, _build_kernel_module

    source = Path(path).resolve()
    if not source.exists():
        raise FileNotFoundError(f"Ea source file not found: {source}")

    cpu = _resolve_target() if target == "native" else target
    version = compiler_version()
    cache = _cache_dir(source, cpu, version)

    if not _is_cached(source, cache):
        cache.mkdir(parents=True, exist_ok=True)
        cache_source = cache / source.name
        try:
            shutil.copy2(str(source), str(cache_source))
            cmd = [str(_ea_binary()), source.name, "--lib"]
            if target != "native":
                cmd.append(f"--target={target}")
            cmd.append(f"--opt-level={opt_level}")
            if avx512:
                cmd.append("--avx512")
            result = subprocess.run(
                cmd, capture_output=True, text=True, timeout=60,
                cwd=str(cache),
            )
            if result.returncode != 0:
                raise CompileError(
                    f"Compilation failed: {source.name}",
                    stderr=result.stderr,
                    exit_code=result.returncode,
                )
        finally:
            if cache_source.exists():
                cache_source.unlink()

    json_path = cache / f"{source.name}.json"
    so_path = cache / f"{source.stem}{_lib_ext()}"
    return _build_kernel_module(json_path, so_path)
