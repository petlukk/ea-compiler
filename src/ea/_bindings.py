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


class _KernelModule:
    """Module-like object returned by ea.load()."""

    def __init__(self, name: str):
        self._name = name

    def __repr__(self):
        funcs = [k for k in self.__dict__ if not k.startswith("_")]
        return f"<ea.Kernel '{self._name}' funcs={funcs}>"


def _build_kernel_module(json_path: Path, so_path: Path):
    """Build module with callable methods from metadata + .so."""
    exports = _parse_metadata(json_path)
    lib = ctypes.CDLL(str(so_path.resolve()))
    module = _KernelModule(json_path.stem.removesuffix(".ea"))
    for export in exports:
        func = _build_function(lib, export)
        setattr(module, export["name"], func)
    return module


def _build_function(lib, export: dict):
    """Build a callable Python function from export metadata."""
    name = export["name"]
    args = export.get("args", [])
    return_type = export.get("return_type")
    collapsed = _detect_collapsed(args)

    c_func = getattr(lib, name)
    argtypes = []
    for arg in args:
        ty = arg["type"]
        if _is_pointer(ty):
            inner = _pointer_inner(ty)
            ct = _ea_type_to_ctype(inner)
            argtypes.append(ctypes.POINTER(ct) if ct else ctypes.c_void_p)
        else:
            ct = _ea_type_to_ctype(ty)
            argtypes.append(ct if ct else ctypes.c_int32)
    c_func.argtypes = argtypes
    c_func.restype = _ea_type_to_ctype(return_type) if return_type else None

    visible_names = []
    for i, arg in enumerate(args):
        if collapsed[i]:
            continue
        direction = arg.get("direction", "in")
        cap = arg.get("cap")
        if direction == "out" and cap is not None:
            continue
        visible_names.append(arg["name"])

    def wrapper(*pos_args, **kwargs):
        for j, val in enumerate(pos_args):
            if j < len(visible_names):
                kwargs[visible_names[j]] = val

        c_args = [None] * len(args)
        out_bufs = {}
        last_array_size = None

        for i, arg in enumerate(args):
            ty = arg["type"]
            arg_name = arg["name"]

            if collapsed[i]:
                ct = _ea_type_to_ctype(ty)
                c_args[i] = ct(last_array_size) if ct else last_array_size
                continue

            direction = arg.get("direction", "in")
            cap = arg.get("cap")

            if direction == "out" and cap is not None:
                inner = _pointer_inner(ty)
                dtype = _ea_type_to_numpy_dtype(inner)
                if cap in kwargs:
                    alloc_size = int(kwargs[cap])
                elif last_array_size is not None:
                    alloc_size = last_array_size
                else:
                    raise ValueError(
                        f"Cannot determine output size for '{arg_name}'"
                    )
                out_buf = np.empty(alloc_size, dtype=dtype)
                ct = _ea_type_to_ctype(inner)
                c_args[i] = out_buf.ctypes.data_as(ctypes.POINTER(ct))
                out_bufs[arg_name] = out_buf
                last_array_size = alloc_size
                continue

            if _is_pointer(ty):
                arr = kwargs.get(arg_name)
                if arr is None:
                    raise TypeError(f"Missing required argument: {arg_name}")
                inner = _pointer_inner(ty)
                expected_dtype = _ea_type_to_numpy_dtype(inner)
                if expected_dtype and arr.dtype != expected_dtype:
                    raise TypeError(
                        f"{arg_name}: expected {expected_dtype}, got {arr.dtype}"
                    )
                ct = _ea_type_to_ctype(inner)
                c_args[i] = arr.ctypes.data_as(ctypes.POINTER(ct))
                last_array_size = arr.size
            else:
                val = kwargs.get(arg_name)
                if val is None:
                    raise TypeError(f"Missing required argument: {arg_name}")
                ct = _ea_type_to_ctype(ty)
                c_args[i] = ct(val) if ct else val

        result = c_func(*c_args)

        if out_bufs and result is None:
            bufs = list(out_bufs.values())
            return bufs[0] if len(bufs) == 1 else tuple(bufs)
        elif out_bufs:
            bufs = list(out_bufs.values())
            return (result, *bufs)
        elif return_type:
            py_type = {"f32": float, "f64": float, "bool": bool}.get(
                return_type
            )
            return py_type(result) if py_type else int(result)
        return None

    wrapper.__name__ = name
    wrapper.__doc__ = f"{name}({', '.join(visible_names)})"
    return wrapper
