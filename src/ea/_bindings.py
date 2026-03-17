"""Runtime ctypes binding generation from .ea.json metadata."""

import ctypes
import json
from pathlib import Path

import numpy as np


class CompileError(RuntimeError):
    """Raised when ea compilation fails."""

    def __init__(self, message: str, stderr: str, exit_code: int):
        super().__init__(message)
        self.stderr = stderr
        self.exit_code = exit_code


_CTYPE_MAP = {
    "f32": ctypes.c_float, "f64": ctypes.c_double,
    "i8": ctypes.c_int8, "i16": ctypes.c_int16,
    "i32": ctypes.c_int32, "i64": ctypes.c_int64,
    "u8": ctypes.c_uint8, "u16": ctypes.c_uint16,
    "u32": ctypes.c_uint32, "u64": ctypes.c_uint64,
    "bool": ctypes.c_bool,
}

_DTYPE_MAP = {
    "f32": np.float32, "f64": np.float64,
    "i8": np.int8, "i16": np.int16,
    "i32": np.int32, "i64": np.int64,
    "u8": np.uint8, "u16": np.uint16,
    "u32": np.uint32, "u64": np.uint64,
}

_LENGTH_NAMES = {"n", "len", "length", "count", "size", "num"}
_INTEGER_TYPES = {"i32", "i64", "u32", "u64"}


def _ea_type_to_ctype(ty):
    return _CTYPE_MAP.get(ty)


def _ea_type_to_numpy_dtype(ty):
    return _DTYPE_MAP.get(ty)


def _is_pointer(ty: str) -> bool:
    return ty.startswith("*")


def _is_mut_pointer(ty: str) -> bool:
    if not ty.startswith("*"):
        return False
    rest = ty[1:].strip()
    if rest.startswith("restrict"):
        rest = rest[len("restrict"):].strip()
    return rest.startswith("mut")


def _pointer_inner(ty: str) -> str:
    """Extract inner type: *f32 -> f32, *mut f32 -> f32, *restrict mut f32 -> f32"""
    rest = ty[1:].strip()
    if rest.startswith("restrict"):
        rest = rest[len("restrict"):].strip()
    if rest.startswith("mut"):
        rest = rest[len("mut"):].strip()
    return rest


def _detect_collapsed(args: list) -> list:
    """Detect length params to auto-fill. Uses has_preceding_pointer flag."""
    collapsed = [False] * len(args)
    has_preceding_pointer = False
    for i, arg in enumerate(args):
        if _is_pointer(arg.get("type", "")):
            has_preceding_pointer = True
        elif (has_preceding_pointer
              and arg["name"] in _LENGTH_NAMES
              and arg.get("type", "") in _INTEGER_TYPES):
            collapsed[i] = True
    return collapsed


def _parse_metadata(json_path: Path) -> list:
    """Parse .ea.json metadata file. Returns list of export dicts."""
    with open(json_path) as f:
        data = json.load(f)
    return data.get("exports", [])
