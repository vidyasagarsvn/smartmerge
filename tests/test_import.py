def test_import():
    import importlib.util
    import smartmerge

    if importlib.util.find_spec("PySide6") is None:
        return

    import smartmerge.ui.file_compare_widget  # noqa: F401
    import smartmerge.ui.open_files_dialog  # noqa: F401
