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
    import sys
    from ea._bindings import CompileError

    source = Path(path).resolve()
    if not source.exists():
        raise FileNotFoundError(f"Ea source file not found: {source}")

    cmd = [str(_ea_binary()), str(source)]
    if lib:
        cmd.append("--lib")
    if emit_asm:
        cmd.append("--emit-asm")
    if emit_llvm:
        cmd.append("--emit-llvm")
    if target != "native":
        cmd.append(f"--target={target}")
    cmd.append(f"--opt-level={opt_level}")
    if avx512:
        cmd.append("--avx512")

    result = subprocess.run(
        cmd, capture_output=True, text=True, timeout=60,
        cwd=str(source.parent),
    )
    if result.returncode != 0:
        raise CompileError(
            f"Compilation failed: {source.name}",
            stderr=result.stderr,
            exit_code=result.returncode,
        )

    stem = source.stem
    if emit_asm:
        return source.with_suffix(".s")
    elif emit_llvm:
        return source.with_suffix(".ll")
    elif lib:
        ext = ".dll" if sys.platform == "win32" else ".so"
        return source.parent / f"{stem}{ext}"
    else:
        return source.with_suffix(".o")


def _resolve_target() -> str:
    """Resolve 'native' to a concrete CPU name for cache keying."""
    raise NotImplementedError
