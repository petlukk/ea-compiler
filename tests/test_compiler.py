import pytest
from unittest.mock import patch, MagicMock
from pathlib import Path


def test_compiler_version_parses_output():
    mock_result = MagicMock()
    mock_result.stdout = "ea 1.7.0\n"
    mock_result.returncode = 0
    with patch("ea._compiler._ea_binary", return_value=Path("/mock/ea")), \
         patch("ea._compiler.subprocess.run", return_value=mock_result):
        from ea._compiler import compiler_version
        assert compiler_version() == "1.7.0"


def test_compile_file_not_found():
    from ea._compiler import compile
    with pytest.raises(FileNotFoundError):
        compile("nonexistent.ea")


def test_resolve_target_native_fallback():
    mock_result = MagicMock()
    mock_result.returncode = 1
    mock_result.stderr = "unknown option"
    with patch("ea._compiler._ea_binary", return_value=Path("/mock/ea")), \
         patch("ea._compiler.subprocess.run", return_value=mock_result):
        from ea._compiler import _resolve_target
        target = _resolve_target()
        assert target and " " not in target


def test_resolve_target_parses_print_target():
    mock_result = MagicMock()
    mock_result.returncode = 0
    mock_result.stdout = "znver4\n"
    with patch("ea._compiler._ea_binary", return_value=Path("/mock/ea")), \
         patch("ea._compiler.subprocess.run", return_value=mock_result):
        from ea._compiler import _resolve_target
        assert _resolve_target() == "znver4"
