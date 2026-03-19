"""Ea SIMD kernel compiler for Python."""

__version__ = "1.10.0"

from ea._compiler import compile, compiler_version
from ea._cache import load, clear_cache
from ea._bindings import CompileError

__all__ = ["load", "compile", "clear_cache", "compiler_version", "CompileError", "__version__"]
