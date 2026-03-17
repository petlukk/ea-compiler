import json
from pathlib import Path
import pytest

SAMPLE_METADATA = {
    "library": "scale.so",
    "exports": [
        {
            "name": "scale",
            "args": [
                {"name": "src", "type": "*restrict f32", "direction": "in", "cap": None, "count": None},
                {"name": "dst", "type": "*mut f32", "direction": "out", "cap": "n", "count": "n"},
                {"name": "factor", "type": "f32", "direction": "in", "cap": None, "count": None},
                {"name": "n", "type": "i32", "direction": "in", "cap": None, "count": None},
            ],
            "return_type": None,
        }
    ],
    "structs": [],
}


def test_parse_metadata(tmp_path):
    json_path = tmp_path / "scale.ea.json"
    json_path.write_text(json.dumps(SAMPLE_METADATA))
    from ea._bindings import _parse_metadata
    exports = _parse_metadata(json_path)
    assert len(exports) == 1
    assert exports[0]["name"] == "scale"
    assert len(exports[0]["args"]) == 4


def test_ea_type_to_ctypes():
    from ea._bindings import _ea_type_to_ctype, _ea_type_to_numpy_dtype
    import ctypes
    import numpy as np
    assert _ea_type_to_ctype("f32") == ctypes.c_float
    assert _ea_type_to_ctype("i32") == ctypes.c_int32
    assert _ea_type_to_ctype("u8") == ctypes.c_uint8
    assert _ea_type_to_ctype("bool") == ctypes.c_bool
    assert _ea_type_to_numpy_dtype("f32") == np.float32
    assert _ea_type_to_numpy_dtype("i32") == np.int32


def test_is_pointer():
    from ea._bindings import _is_pointer, _is_mut_pointer, _pointer_inner
    assert _is_pointer("*f32")
    assert _is_pointer("*mut f32")
    assert _is_pointer("*restrict f32")
    assert not _is_pointer("f32")
    assert _is_mut_pointer("*mut f32")
    assert _is_mut_pointer("*restrict mut f32")
    assert not _is_mut_pointer("*f32")
    assert _pointer_inner("*f32") == "f32"
    assert _pointer_inner("*mut f32") == "f32"
    assert _pointer_inner("*restrict mut f32") == "f32"


def test_length_collapsing():
    from ea._bindings import _detect_collapsed
    args = [{"name": "src", "type": "*f32"}, {"name": "n", "type": "i32"}]
    assert _detect_collapsed(args) == [False, True]


def test_length_collapsing_non_adjacent():
    from ea._bindings import _detect_collapsed
    args = [
        {"name": "data", "type": "*f32"},
        {"name": "factor", "type": "f32"},
        {"name": "n", "type": "i32"},
    ]
    assert _detect_collapsed(args) == [False, False, True]


def test_length_collapsing_not_without_pointer():
    from ea._bindings import _detect_collapsed
    args = [{"name": "factor", "type": "f32"}, {"name": "n", "type": "i32"}]
    assert _detect_collapsed(args) == [False, False]
