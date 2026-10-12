"""Fuzz the public-file scanner's text decoding, quick-start parsing and safety patterns."""

from __future__ import annotations

import sys

from scripts import check_public_files as public_files


def test_one_input(data: bytes) -> None:
    text, failure = public_files.decoded_text(data)
    # A decoded text never carries a failure, and a failure never carries text.
    # Explicit raises, because python -O strips assert statements.
    if text is not None and failure is not None:
        raise AssertionError(f"decoded text also carried failure {failure!r}")
    if failure not in (None, "unclassifiable-text", "undecodable-text"):
        raise AssertionError(f"unexpected failure {failure!r}")

    readme = data.decode("utf-8", errors="replace")
    commands = public_files.quick_start_commands(readme)
    if not all(command and command == command.strip() for command in commands):
        raise AssertionError(f"blank or unstripped quick-start command in {commands!r}")

    # Every safety pattern must finish on arbitrary text; a pattern that
    # backtracks without bound shows up here as a timeout.
    for _, pattern in public_files.TEXT_RULES:
        pattern.search(readme)
    public_files.PRIVATE_CONFIG.search(readme)
    public_files.CLIENT_DATA_PATH.search(readme)


if __name__ == "__main__":
    import atheris

    atheris.instrument_all()
    atheris.Setup(sys.argv, test_one_input)
    atheris.Fuzz()
