"""Configuration module for SmartMerge."""

import json
from pathlib import Path
from typing import Optional

# Default configuration values
DEFAULT_THEME = "dark"
DEFAULT_FONT_SIZE = 12
DEFAULT_FONT_FAMILY = "Courier New"
DEFAULT_TAB_WIDTH = 4
HIGHLIGHT_COLORS = {
    "modified": "#FFFF00",
    "added": "#00FF00",
    "deleted": "#FF0000",
}


def load_config(config_file: Path) -> dict:
    """Load configuration from JSON file."""
    if config_file.exists():
        with open(config_file, "r") as f:
            return json.load(f)
    return {}


def save_config(config: dict, config_file: Path) -> None:
    """Save configuration to JSON file."""
    config_file.parent.mkdir(parents=True, exist_ok=True)
    with open(config_file, "w") as f:
        json.dump(config, f, indent=2)


def get_setting(config: dict, key: str, default: Optional[str] = None) -> Optional[str]:
    """Get a configuration setting with a default fallback."""
    return config.get(key, default)
