"""Runtime ctypes binding generation from .ea.json metadata."""


class CompileError(RuntimeError):
    """Raised when ea compilation fails."""

    def __init__(self, message: str, stderr: str, exit_code: int):
        super().__init__(message)
        self.stderr = stderr
        self.exit_code = exit_code
