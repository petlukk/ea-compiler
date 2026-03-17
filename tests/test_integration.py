import sys
import shutil
from pathlib import Path

import numpy as np
import pytest

try:
    from ea._compiler import _ea_binary
    _ea_binary()
    EA_AVAILABLE = True
except (RuntimeError, OSError):
    EA_AVAILABLE = False

pytestmark = pytest.mark.skipif(not EA_AVAILABLE, reason="ea binary not available")

KERNEL_DIR = Path(__file__).parent / "kernels"


class TestLoad:
    def test_scale_kernel(self, tmp_path):
        shutil.copy(KERNEL_DIR / "scale.ea", tmp_path / "scale.ea")
        import ea
        kernel = ea.load(str(tmp_path / "scale.ea"))
        src = np.array([1.0, 2.0, 3.0, 4.0], dtype=np.float32)
        dst = np.empty_like(src)
        kernel.scale(src, dst, factor=3.0)
        np.testing.assert_array_almost_equal(dst, [3.0, 6.0, 9.0, 12.0])

    def test_dot_product_kernel(self, tmp_path):
        shutil.copy(KERNEL_DIR / "dot.ea", tmp_path / "dot.ea")
        import ea
        kernel = ea.load(str(tmp_path / "dot.ea"))
        a = np.array([1.0, 2.0, 3.0], dtype=np.float32)
        b = np.array([4.0, 5.0, 6.0], dtype=np.float32)
        result = kernel.dot(a, b)
        assert abs(result - 32.0) < 1e-5

    def test_cache_created(self, tmp_path):
        shutil.copy(KERNEL_DIR / "scale.ea", tmp_path / "scale.ea")
        import ea
        ea.load(str(tmp_path / "scale.ea"))
        assert (tmp_path / "__eacache__").exists()

    def test_second_load_uses_cache(self, tmp_path):
        shutil.copy(KERNEL_DIR / "scale.ea", tmp_path / "scale.ea")
        import ea
        ea.load(str(tmp_path / "scale.ea"))
        assert (tmp_path / "__eacache__").exists()
        kernel2 = ea.load(str(tmp_path / "scale.ea"))
        src = np.array([1.0, 2.0], dtype=np.float32)
        dst = np.empty_like(src)
        kernel2.scale(src, dst, factor=2.0)
        np.testing.assert_array_almost_equal(dst, [2.0, 4.0])

    def test_clear_cache(self, tmp_path):
        shutil.copy(KERNEL_DIR / "scale.ea", tmp_path / "scale.ea")
        import ea
        ea.load(str(tmp_path / "scale.ea"))
        ea.clear_cache(tmp_path)
        assert not (tmp_path / "__eacache__").exists()

    def test_compile_error_raises(self, tmp_path):
        bad_file = tmp_path / "bad.ea"
        bad_file.write_text("this is not valid ea code")
        import ea
        with pytest.raises(ea.CompileError):
            ea.load(str(bad_file))

    def test_file_not_found(self):
        import ea
        with pytest.raises(FileNotFoundError):
            ea.load("nonexistent.ea")

    def test_wrong_dtype_raises(self, tmp_path):
        shutil.copy(KERNEL_DIR / "scale.ea", tmp_path / "scale.ea")
        import ea
        kernel = ea.load(str(tmp_path / "scale.ea"))
        src = np.array([1.0, 2.0], dtype=np.float64)
        dst = np.empty(2, dtype=np.float32)
        with pytest.raises(TypeError):
            kernel.scale(src, dst, factor=2.0)


class TestCompile:
    def test_compile_produces_so(self, tmp_path):
        shutil.copy(KERNEL_DIR / "scale.ea", tmp_path / "scale.ea")
        import ea
        result = ea.compile(str(tmp_path / "scale.ea"))
        ext = ".dll" if sys.platform == "win32" else ".so"
        assert result.suffix == ext
        assert result.exists()

    def test_compile_file_not_found(self):
        import ea
        with pytest.raises(FileNotFoundError):
            ea.compile("nonexistent.ea")


class TestMisc:
    def test_version(self):
        import ea
        assert ea.__version__

    def test_compiler_version(self):
        import ea
        v = ea.compiler_version()
        assert v
        parts = v.split(".")
        assert len(parts) >= 2
