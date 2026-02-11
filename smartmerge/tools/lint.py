import subprocess
import sys


def main() -> None:
    try:
        result = subprocess.run(["ruff", "check", "."], check=False)
    except FileNotFoundError:
        print("ruff is not installed. Install dev dependencies: uv sync --extra dev")
        sys.exit(1)

    sys.exit(result.returncode)
