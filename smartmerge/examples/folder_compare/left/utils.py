"""Utilities for SmartMerge."""


def format_size(bytes_value: int) -> str:
    """Format bytes to human-readable size."""
    for unit in ["B", "KB", "MB", "GB"]:
        if bytes_value < 1024.0:
            return f"{bytes_value:.1f} {unit}"
        bytes_value /= 1024.0
    return f"{bytes_value:.1f} TB"


def normalize_path(path_str: str) -> str:
    """Normalize path separators."""
    return path_str.replace("\\", "/")


def truncate_text(text: str, max_length: int = 80) -> str:
    """Truncate text to max length."""
    if len(text) > max_length:
        return text[: max_length - 3] + "..."
    return text
