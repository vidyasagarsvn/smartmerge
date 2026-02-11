from typing import Iterable, List


def _format_line(index: int, token: str) -> str:
    return f"Line {index:02d}: {token}"


def _common_tokens() -> List[str]:
    return [
        "alpha",
        "beta",
        "gamma",
        "common-A",
        "delta",
        "epsilon",
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
    base = _splice_extras(_common_tokens())
    tokens = base + _extra_tokens()[3:]
    return [_format_line(idx + 1, token) for idx, token in enumerate(tokens)]


def emit(lines: Iterable[str]) -> None:
    for line in lines:
        print(line)


if __name__ == "__main__":
    emit(build_lines())


def _extra_tokens() -> List[str]:
    return [f"extra-{idx}" for idx in range(1, 21)]


def _splice_extras(tokens: List[str]) -> List[str]:
    with_extras = tokens.copy()
    inserts = [
        (6, "extra-1"),
        (11, "extra-2"),
        (18, "extra-3"),
    ]
    for offset, extra in inserts:
        with_extras.insert(offset, extra)
    return with_extras
