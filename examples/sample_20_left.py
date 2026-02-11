from typing import Iterable, List


def _format_line(index: int, token: str) -> str:
    return f"Line {index:02d}: {token}"


def _common_tokens() -> List[str]:
    return ["alpha", "beta", "gamma", "common-A", "delta", "epsilon"]


def _left_only_tokens() -> List[str]:
    return [
        "common-B",
        "zeta",
        "eta",
        "common-C",
        "theta",
        "iota",
        "kappa",
        "common-D",
        "lambda",
        "mu",
        "common-E",
        "nu",
        "xi",
        "omega",
    ]


def build_lines() -> List[str]:
    tokens = _common_tokens() + _left_only_tokens()
    return [_format_line(idx + 1, token) for idx, token in enumerate(tokens)]


def emit(lines: Iterable[str]) -> None:
    for line in lines:
        print(line)


if __name__ == "__main__":
    emit(build_lines())
