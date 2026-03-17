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
