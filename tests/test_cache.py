import sys
import time
from pathlib import Path
import pytest


def test_cache_dir_created_relative_to_source(tmp_path):
    from ea._cache import _cache_dir
    ea_file = tmp_path / "kernel.ea"
    ea_file.touch()
    cache = _cache_dir(ea_file, "znver4", "1.7.0")
    assert cache == tmp_path / "__eacache__" / "znver4-1.7.0"


def test_cache_is_stale_when_no_cache(tmp_path):
    from ea._cache import _is_cached
    ea_file = tmp_path / "kernel.ea"
    ea_file.write_text("export func f() {}")
    cache = tmp_path / "__eacache__" / "znver4-1.7.0"
    assert not _is_cached(ea_file, cache)


def test_cache_is_fresh_when_so_newer(tmp_path):
    from ea._cache import _is_cached
    ea_file = tmp_path / "kernel.ea"
    ea_file.write_text("export func f() {}")
    cache = tmp_path / "__eacache__" / "znver4-1.7.0"
    cache.mkdir(parents=True)
    ext = ".dll" if sys.platform == "win32" else ".so"
    so_file = cache / f"kernel{ext}"
    time.sleep(0.05)
    so_file.write_bytes(b"\x00")
    assert _is_cached(ea_file, cache)


def test_clear_cache_removes_eacache(tmp_path):
    from ea._cache import clear_cache
    cache = tmp_path / "__eacache__" / "znver4-1.7.0"
    cache.mkdir(parents=True)
    (cache / "kernel.so").write_bytes(b"\x00")
    clear_cache(tmp_path)
    assert not (tmp_path / "__eacache__").exists()
