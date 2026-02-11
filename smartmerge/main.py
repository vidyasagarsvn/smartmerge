import sys
from pathlib import Path
from PySide6.QtWidgets import (
    QApplication,
    QMainWindow,
    QMenuBar,
    QStatusBar,
    QFontDialog,
    QStackedWidget,
    QFileDialog,
)
from PySide6.QtCore import QSettings
from PySide6.QtGui import QAction, QIcon, QFont
from importlib.resources import files, as_file
from smartmerge.ui.file_compare_widget import FileCompareWidget
from smartmerge.ui.folder_compare_widget import FolderCompareWidget
from smartmerge.ui.open_files_dialog import OpenFilesDialog


def _get_platform_shortcut(linux_windows: str, macos: str) -> str:
    """Return platform-specific keyboard shortcut.

    On macOS, Qt uses 'Meta' for the Command key which displays as ⌘.
    """
    if sys.platform == "darwin":  # macOS
        # Replace "Cmd" with "Meta" for Qt compatibility
        return macos.replace("Cmd", "Meta")
    return linux_windows


class MainWindow(QMainWindow):
    def __init__(self):
        super().__init__()
        self.setWindowTitle("SmartMerge - File & Folder Compare")
        self._set_app_icon()
        self.resize(1200, 700)

        # Initialize state before creating UI
        self.navigation_stack: list[tuple[Path, Path]] = []  # For breadcrumb navigation
        self.viewing_from_folder: bool = False  # Track if we navigated from folder view
        self.back_action: QAction | None = (
            None  # Reference to back action for enabling/disabling
        )

        # Create stacked widget to switch between file and folder views
        self.stacked_widget = QStackedWidget()

        # File comparison widget
        self.file_compare = FileCompareWidget()
        self.stacked_widget.addWidget(self.file_compare)

        # Folder comparison widget
        self.folder_compare = FolderCompareWidget()
        self.folder_compare.folder_selected.connect(self._on_folder_selected)
        self.folder_compare.file_selected.connect(self._on_file_selected)
        self.stacked_widget.addWidget(self.folder_compare)

        if sys.platform == "darwin":
            default_font = QFont("Menlo", 14)
        else:
            default_font = QFont("Courier New", 14)
        self.file_compare.set_font(default_font)
        self.folder_compare.set_font(default_font)

        self.setCentralWidget(self.stacked_widget)
        self._create_menu()
        self.statusBar = QStatusBar()
        self.setStatusBar(self.statusBar)
        self.settings = QSettings("SmartMerge", "SmartMerge")
        self.last_left_path = self.settings.value("last_left_path", "", type=str)
        self.last_right_path = self.settings.value("last_right_path", "", type=str)

    def _set_app_icon(self):
        try:
            icon_resource = files("smartmerge.resources").joinpath("app_icon.svg")
            with as_file(icon_resource) as icon_path:
                self.setWindowIcon(QIcon(str(icon_path)))
        except Exception:
            pass

    def _create_menu(self):
        menubar = QMenuBar(self)
        file_menu = menubar.addMenu("File")
        open_files_action = QAction("Open Files", self)
        open_folders_action = QAction("Open Folders", self)
        self.back_action = QAction("Back to Folder View", self)
        exit_action = QAction("Exit", self)
        open_files_action.setShortcut(_get_platform_shortcut("Ctrl+O", "Cmd+O"))
        open_folders_action.setShortcut(
            _get_platform_shortcut("Ctrl+Shift+O", "Cmd+Shift+O")
        )
        self.back_action.setShortcut(_get_platform_shortcut("Ctrl+B", "Cmd+B"))
        exit_action.setShortcut(_get_platform_shortcut("Ctrl+Q", "Cmd+Q"))
        open_files_action.triggered.connect(self.open_files_dialog)
        open_folders_action.triggered.connect(self.open_folders_dialog)
        self.back_action.triggered.connect(self.back_to_folder_view)
        exit_action.triggered.connect(self.close)
        file_menu.addAction(open_files_action)
        file_menu.addAction(open_folders_action)
        file_menu.addSeparator()
        file_menu.addAction(self.back_action)
        file_menu.addSeparator()
        file_menu.addAction(exit_action)
        self.back_action.setEnabled(False)  # Disabled by default

        edit_menu = menubar.addMenu("Edit")
        font_action = QAction("Font", self)
        font_action.setShortcut(_get_platform_shortcut("Ctrl+Shift+F", "Cmd+Shift+F"))
        font_action.triggered.connect(self.open_font_dialog)
        edit_menu.addAction(font_action)

        # Options menu
        options_menu = menubar.addMenu("Options")

        # Engine submenu (under Options)
        engine_menu = options_menu.addMenu("Engine")
        self.myers_action = QAction("Myers Diff Algorithm", self)
        self.myers_action.setCheckable(True)
        self.myers_action.setChecked(False)
        self.myers_action.triggered.connect(lambda: self._set_diff_engine("myers"))

        self.smart_action = QAction("Smart Block Diff", self)
        self.smart_action.setCheckable(True)
        self.smart_action.setChecked(True)  # Currently active engine
        self.smart_action.triggered.connect(lambda: self._set_diff_engine("smart"))

        engine_menu.addAction(self.myers_action)
        engine_menu.addAction(self.smart_action)

        self.setMenuBar(menubar)

    def open_font_dialog(self):
        if self.stacked_widget.currentIndex() == 0:  # File comparison
            current_font = self.file_compare.left_text.font()
        else:  # Folder comparison
            current_font = self.folder_compare.table.font()

        result = QFontDialog.getFont(current_font, self, "Select Font")
        if isinstance(result, tuple) and len(result) == 2:
            first, second = result
            if isinstance(first, bool):
                ok, font = first, second
            else:
                font, ok = first, second
        else:
            font, ok = result, True

        if ok:
            self.file_compare.set_font(font)
            self.folder_compare.set_font(font)

    def open_files_dialog(self):
        dialog = OpenFilesDialog(
            self,
            initial_left_path=self.last_left_path,
            initial_right_path=self.last_right_path,
        )
        from PySide6.QtWidgets import QDialog

        if dialog.exec() == QDialog.Accepted:
            left_path, right_path = dialog.get_paths()
            if left_path:
                self.file_compare.load_left_file(left_path)
                self.statusBar.showMessage(f"Loaded left file: {left_path}")
                self.last_left_path = left_path
                self.settings.setValue("last_left_path", left_path)
            if right_path:
                self.file_compare.load_right_file(right_path)
                self.statusBar.showMessage(f"Loaded right file: {right_path}")
                self.last_right_path = right_path
                self.settings.setValue("last_right_path", right_path)

            # Switch to file comparison view
            self.stacked_widget.setCurrentIndex(0)

    def open_folders_dialog(self):
        """Open dialog to select two folders for comparison."""
        left_path = QFileDialog.getExistingDirectory(
            self, "Select Left Folder", self.last_left_path or str(Path.home())
        )
        if not left_path:
            return

        right_path = QFileDialog.getExistingDirectory(
            self, "Select Right Folder", self.last_right_path or str(Path.home())
        )
        if not right_path:
            return

        left_path_obj = Path(left_path)
        right_path_obj = Path(right_path)

        # Update last paths
        self.last_left_path = left_path
        self.last_right_path = right_path
        self.settings.setValue("last_left_path", left_path)
        self.settings.setValue("last_right_path", right_path)

        # Clear navigation stack and load folder comparison
        self.navigation_stack = []
        self.folder_compare.set_paths(left_path_obj, right_path_obj)
        self.stacked_widget.setCurrentIndex(1)
        self.statusBar.showMessage(f"Comparing folders: {left_path} ↔ {right_path}")

    def _on_folder_selected(self, left_path: Path, right_path: Path):
        """Handle navigation into a subfolder."""
        # Push current state to navigation stack
        if self.folder_compare.left_path and self.folder_compare.right_path:
            self.navigation_stack.append(
                (self.folder_compare.left_path, self.folder_compare.right_path)
            )

        # Update folder view
        self.folder_compare.set_paths(left_path, right_path)
        self.statusBar.showMessage(f"In: {left_path.name} ↔ {right_path.name}")

    def _on_file_selected(self, left_path: Path, right_path: Path):
        """Handle request to compare two files."""
        # Load files into file comparison widget
        self.file_compare.load_left_file(str(left_path))
        self.file_compare.load_right_file(str(right_path))

        # Mark that we're viewing from folder context
        self.viewing_from_folder = True
        if self.back_action:
            self.back_action.setEnabled(True)

        # Switch to file comparison view
        self.stacked_widget.setCurrentIndex(0)
        self.statusBar.showMessage(f"Comparing: {left_path.name} ↔ {right_path.name}")

    def back_to_folder_view(self):
        """Return to the folder comparison view."""
        if self.viewing_from_folder:
            self.stacked_widget.setCurrentIndex(1)
            self.viewing_from_folder = False
            if self.back_action:
                self.back_action.setEnabled(False)
            if self.folder_compare.left_path and self.folder_compare.right_path:
                self.statusBar.showMessage(
                    f"Back to: {self.folder_compare.left_path.name} ↔ "
                    f"{self.folder_compare.right_path.name}"
                )

    def _set_diff_engine(self, engine_name: str):
        """Set the diff engine and update UI."""
        # Update menu checkmarks
        if engine_name == "myers":
            self.myers_action.setChecked(True)
            self.smart_action.setChecked(False)
        elif engine_name == "smart":
            self.myers_action.setChecked(False)
            self.smart_action.setChecked(True)

        # Set engine on file compare widget
        self.file_compare.set_diff_engine(engine_name)
        self.statusBar.showMessage(
            f"Switched to {engine_name.capitalize()} Diff Engine"
        )


def main():
    app = QApplication(sys.argv)
    window = MainWindow()
    window.show()
    sys.exit(app.exec())


if __name__ == "__main__":
    main()
