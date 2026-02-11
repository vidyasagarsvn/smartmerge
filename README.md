# smartmerge
A lightweight, cross-platform file and folder diff/merge tool with visual highlighting and intelligent conflict resolution.

## Project Structure

```
smartmerge_python/
│
├── smartmerge/                # Main package directory
│   ├── __init__.py
│   ├── __main__.py            # Allows running with `python -m smartmerge`
│   ├── main.py                # Main entry point
│   ├── ui/                    # UI components (dialogs, widgets)
│   │   ├── __init__.py
│   │   ├── file_compare_widget.py
│   │   └── open_files_dialog.py
│   ├── core/                  # Core logic (diff, merge, utils)
│   │   ├── __init__.py
│   └── resources/             # Images, icons, etc.
│
├── examples/                  # Example files for testing
├── tests/                     # Unit tests
├── requirements.txt
├── README.md
├── LICENSE
└── setup.py                   # Or pyproject.toml for packaging
```

## Running the App

From the root directory:

```
uv venv
source .venv/bin/activate
uv pip install pyside6
python -m smartmerge
```

## Features
- File and folder comparison
- Merge functionality
- Visual highlighting
- Cross-platform support

## Contributing
Contributions are welcome! Please open issues or pull requests.
