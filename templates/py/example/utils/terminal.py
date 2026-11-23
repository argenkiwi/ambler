"""Terminal and standard I/O utilities."""


def prompt(message: str) -> str | None:
    """Prompts with ``message`` on stdout and reads a line from stdin.

    Returns ``None`` at end of input. The trailing newline is stripped.

    Raises:
        OSError: if stdout cannot be written or stdin cannot be read.
    """
    try:
        return input(message)
    except EOFError:
        return None
