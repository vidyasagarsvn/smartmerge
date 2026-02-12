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
    QMessageBox,
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
        self.current_theme: str = "light"  # Track current theme (light or dark)

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
            default_font = QFont("Monaco", 13)
        else:
            default_font = QFont("Courier New", 13)
        self.file_compare.set_font(default_font)
        self.folder_compare.set_font(default_font)

        self.setCentralWidget(self.stacked_widget)
        self._create_menu()
        self.statusBar = QStatusBar()
        self.setStatusBar(self.statusBar)
        self.settings = QSettings("SmartMerge", "SmartMerge")
        self.last_left_path = self.settings.value("last_left_path", "", type=str)
        self.last_right_path = self.settings.value("last_right_path", "", type=str)

        # Restore window geometry if saved
        geometry = self.settings.value("window_geometry", None)
        if geometry:
            self.restoreGeometry(geometry)

        # Apply default light theme
        self._apply_light_theme()

    def _set_app_icon(self):
        try:
            icon_resource = files("smartmerge.resources").joinpath("app_icon.svg")
            with as_file(icon_resource) as icon_path:
                self.setWindowIcon(QIcon(str(icon_path)))
        except Exception:
            pass

    def _create_menu(self):
        menubar = self.menuBar()
        file_menu = menubar.addMenu("File")
        open_files_action = QAction("Open Files", self)
        open_folders_action = QAction("Open Folders", self)
        self.back_action = QAction("Back to Folder View", self)
        exit_action = QAction("Exit", self)
        open_files_action.setShortcut(_get_platform_shortcut("Ctrl+O", "Cmd+O"))
        open_folders_action.setShortcut(
            _get_platform_shortcut("Ctrl+Shift+O", "Cmd+Shift+O")
        )
        self.back_action.setShortcut("Escape")
        exit_action.setShortcut(_get_platform_shortcut("Ctrl+Q", "Cmd+Q"))
        open_files_action.triggered.connect(self.open_files_dialog)
        open_folders_action.triggered.connect(self.open_folders_dialog)
        self.back_action.triggered.connect(self.back_to_folder_view)
        exit_action.triggered.connect(self.close)

        # Make main actions bold
        bold_font = open_files_action.font()
        bold_font.setBold(True)
        open_files_action.setFont(bold_font)
        open_folders_action.setFont(bold_font)

        file_menu.addAction(open_files_action)
        file_menu.addAction(open_folders_action)
        file_menu.addSeparator()
        file_menu.addAction(self.back_action)
        file_menu.addSeparator()
        file_menu.addAction(exit_action)
        self.back_action.setEnabled(False)  # Disabled by default

        edit_menu = menubar.addMenu("Edit")

        # Undo/Redo actions - make bold for visibility
        self.undo_action = QAction("Undo", self)
        self.undo_action.setShortcut(_get_platform_shortcut("Ctrl+Z", "Cmd+Z"))
        self.undo_action.triggered.connect(self._undo)
        self.undo_action.setEnabled(False)
        undo_font = self.undo_action.font()
        undo_font.setBold(True)
        self.undo_action.setFont(undo_font)
        edit_menu.addAction(self.undo_action)

        self.redo_action = QAction("Redo", self)
        self.redo_action.setShortcut(
            _get_platform_shortcut("Ctrl+Shift+Z", "Cmd+Shift+Z")
        )
        self.redo_action.triggered.connect(self._redo)
        self.redo_action.setEnabled(False)
        redo_font = self.redo_action.font()
        redo_font.setBold(True)
        self.redo_action.setFont(redo_font)
        edit_menu.addAction(self.redo_action)

        edit_menu.addSeparator()

        # Save actions - make bold for visibility
        save_action = QAction("Save Left File", self)
        save_action.setShortcut(_get_platform_shortcut("Ctrl+S", "Cmd+S"))
        save_action.triggered.connect(self._save)
        save_font = save_action.font()
        save_font.setBold(True)
        save_action.setFont(save_font)
        edit_menu.addAction(save_action)

        save_all_action = QAction("Save All Files", self)
        save_all_action.setShortcut(
            _get_platform_shortcut("Ctrl+Shift+S", "Cmd+Shift+S")
        )
        save_all_action.triggered.connect(self._save_all)
        save_all_font = save_all_action.font()
        save_all_font.setBold(True)
        save_all_action.setFont(save_all_font)
        edit_menu.addAction(save_all_action)

        edit_menu.addSeparator()

        font_action = QAction("Font...", self)
        font_action.setShortcut(_get_platform_shortcut("Ctrl+Shift+F", "Cmd+Shift+F"))
        font_action.triggered.connect(self.open_font_dialog)
        edit_menu.addAction(font_action)

        edit_menu.addSeparator()

        # Search action - make bold for visibility
        search_action = QAction("Find...", self)
        search_action.setShortcut(_get_platform_shortcut("Ctrl+F", "Cmd+F"))
        search_action.triggered.connect(self._open_search)
        search_font = search_action.font()
        search_font.setBold(True)
        search_action.setFont(search_font)
        edit_menu.addAction(search_action)

        # Go to line action (disabled - requires inline editing)
        # go_to_line_action = QAction("Go to Line...", self)
        # go_to_line_action.setShortcut(_get_platform_shortcut("Ctrl+G", "Cmd+G"))
        # go_to_line_action.triggered.connect(self._open_go_to_line)
        # edit_menu.addAction(go_to_line_action)

        # Help menu
        help_menu = menubar.addMenu("Help")
        about_action = QAction("About SmartMerge", self)
        about_action.triggered.connect(self._show_about)
        help_menu.addAction(about_action)

        # Options menu
        options_menu = menubar.addMenu("Options")

        # Theme submenu (under Options)
        theme_menu = options_menu.addMenu("Theme")
        self.light_mode_action = QAction("Light Mode", self)
        self.light_mode_action.setCheckable(True)
        self.light_mode_action.setChecked(True)  # Default theme
        self.light_mode_action.triggered.connect(lambda: self._set_theme("light"))

        self.dark_mode_action = QAction("Dark Mode", self)
        self.dark_mode_action.setCheckable(True)
        self.dark_mode_action.setChecked(False)
        self.dark_mode_action.triggered.connect(lambda: self._set_theme("dark"))

        theme_menu.addAction(self.light_mode_action)
        theme_menu.addAction(self.dark_mode_action)

        navigate_menu = menubar.addMenu("Navigate")
        next_change_action = QAction("Next Change", self)
        next_change_action.setShortcut(_get_platform_shortcut("Alt+Down", "Alt+Down"))
        next_change_action.triggered.connect(self._next_change)
        prev_change_action = QAction("Previous Change", self)
        prev_change_action.setShortcut(_get_platform_shortcut("Alt+Up", "Alt+Up"))
        prev_change_action.triggered.connect(self._previous_change)
        navigate_menu.addAction(next_change_action)
        navigate_menu.addAction(prev_change_action)
        navigate_menu.addSeparator()
        copy_to_right_action = QAction("Copy to Right", self)
        copy_to_right_action.setShortcut(
            _get_platform_shortcut("Alt+Right", "Alt+Right")
        )
        copy_to_right_action.triggered.connect(self._copy_to_right)
        copy_to_left_action = QAction("Copy to Left", self)
        copy_to_left_action.setShortcut(_get_platform_shortcut("Alt+Left", "Alt+Left"))
        copy_to_left_action.triggered.connect(self._copy_to_left)
        navigate_menu.addAction(copy_to_right_action)
        navigate_menu.addAction(copy_to_left_action)

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

    def open_font_dialog(self):
        if self.stacked_widget.currentIndex() == 0:  # File comparison
            current_font = self.file_compare.left_text.font()
        else:  # Folder comparison
            current_font = self.folder_compare.table.font()

        dialog = QFontDialog(current_font, self)
        dialog.setWindowTitle("Select Font")
        # Center dialog on parent window
        dialog.move(
            self.x() + (self.width() - dialog.width()) // 2,
            self.y() + (self.height() - dialog.height()) // 2,
        )
        ok = dialog.exec() == QFontDialog.Accepted
        font = dialog.selectedFont() if ok else current_font

        if ok:
            self.file_compare.set_font(font)
            self.folder_compare.set_font(font)

    def _next_change(self):
        if self.stacked_widget.currentIndex() == 0:
            self.file_compare.next_change()

    def _previous_change(self):
        if self.stacked_widget.currentIndex() == 0:
            self.file_compare.previous_change()

    def _copy_to_right(self):
        if self.stacked_widget.currentIndex() == 0:
            self.file_compare.copy_to_right()
            self._update_undo_redo_state()

    def _copy_to_left(self):
        if self.stacked_widget.currentIndex() == 0:
            self.file_compare.copy_to_left()
            self._update_undo_redo_state()

    def _undo(self):
        if self.stacked_widget.currentIndex() == 0:
            self.file_compare.undo()
            self._update_undo_redo_state()

    def _redo(self):
        if self.stacked_widget.currentIndex() == 0:
            self.file_compare.redo()
            self._update_undo_redo_state()

    def _update_undo_redo_state(self):
        """Update the enabled state of undo/redo actions."""
        if self.stacked_widget.currentIndex() == 0:
            self.undo_action.setEnabled(self.file_compare.can_undo())
            self.redo_action.setEnabled(self.file_compare.can_redo())

    def _save(self):
        """Save the currently active file (left or right based on focus)."""
        if self.stacked_widget.currentIndex() == 0:
            # In file compare view, save both if modified
            left_saved = False
            right_saved = False
            if self.file_compare.is_left_modified:
                left_saved = self.file_compare.save_left_file()
            if self.file_compare.is_right_modified:
                right_saved = self.file_compare.save_right_file()

            if left_saved or right_saved:
                files_saved = []
                if left_saved:
                    files_saved.append("Left file")
                if right_saved:
                    files_saved.append("Right file")
                self.statusBar.showMessage(f"Saved: {', '.join(files_saved)}")
            else:
                self.statusBar.showMessage("No modified files to save")

    def _save_all(self):
        """Save all modified files."""
        if self.stacked_widget.currentIndex() == 0:
            left_saved = False
            right_saved = False
            if self.file_compare.is_left_modified:
                left_saved = self.file_compare.save_left_file()
            if self.file_compare.is_right_modified:
                right_saved = self.file_compare.save_right_file()

            if left_saved or right_saved:
                files_saved = []
                if left_saved:
                    files_saved.append("Left file")
                if right_saved:
                    files_saved.append("Right file")
                self.statusBar.showMessage(f"Saved all: {', '.join(files_saved)}")
            else:
                self.statusBar.showMessage("No modified files to save")

    def open_files_dialog(self):
        dialog = OpenFilesDialog(
            self,
            initial_left_path=self.last_left_path,
            initial_right_path=self.last_right_path,
            theme=self.current_theme,
        )
        from PySide6.QtWidgets import QDialog

        if dialog.exec() == QDialog.Accepted:
            left_path, right_path = dialog.get_paths()
            if left_path:
                self.file_compare.load_left_file(left_path)
                self.last_left_path = left_path
                self.settings.setValue("last_left_path", left_path)
            if right_path:
                self.file_compare.load_right_file(right_path)
                self.last_right_path = right_path
                self.settings.setValue("last_right_path", right_path)

            # Update status bar with comparison info
            status = self.file_compare.get_status_info()
            self.statusBar.showMessage(status)

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
        # Check for unsaved changes
        if self._has_unsaved_changes():
            if not self._show_unsaved_changes_dialog("Back to Folder View"):
                return  # User cancelled

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

    def _set_theme(self, theme_name: str):
        """Set the application theme (light or dark mode)."""
        # Update menu checkmarks
        if theme_name == "light":
            self.light_mode_action.setChecked(True)
            self.dark_mode_action.setChecked(False)
            self.current_theme = "light"
            self.file_compare.set_theme("light")
            self.folder_compare.set_theme("light")
            self._apply_light_theme()
        elif theme_name == "dark":
            self.light_mode_action.setChecked(False)
            self.dark_mode_action.setChecked(True)
            self.current_theme = "dark"
            self.file_compare.set_theme("dark")
            self.folder_compare.set_theme("dark")
            self._apply_dark_theme()

        self.statusBar.showMessage(f"Switched to {theme_name.capitalize()} Mode")

    def _apply_light_theme(self):
        """Apply light theme stylesheet."""
        light_stylesheet = """
            QMainWindow { background-color: #ffffff; color: #000000; }
            QMenuBar { background-color: #f5f5f5; color: #000000; border-bottom: 1px solid #e0e0e0; }
            QMenuBar::item:selected { background-color: #e8e8e8; }
            QMenu { background-color: #ffffff; color: #000000; border: 1px solid #d0d0d0; }
            QMenu::item:selected { background-color: #e8e8e8; }
            QTextEdit { background-color: #ffffff; color: #000000; border: 1px solid #d0d0d0; }
            QStatusBar { background-color: #f5f5f5; color: #000000; border-top: 1px solid #e0e0e0; }
            QLineEdit { background-color: #ffffff; color: #000000; border: 1px solid #d0d0d0; }
            QPushButton { background-color: #f0f0f0; color: #000000; border: 1px solid #d0d0d0; border-radius: 4px; padding: 5px; }
            QPushButton:hover { background-color: #e0e0e0; }
            QPushButton:pressed { background-color: #d0d0d0; }
        """
        self.setStyleSheet(light_stylesheet)

    def _apply_dark_theme(self):
        """Apply dark theme stylesheet."""
        dark_stylesheet = """
            QMainWindow { background-color: #1e1e1e; color: #e0e0e0; }
            QMenuBar { background-color: #2d2d2d; color: #e0e0e0; border-bottom: 1px solid #3d3d3d; }
            QMenuBar::item:selected { background-color: #3d3d3d; }
            QMenu { background-color: #2d2d2d; color: #e0e0e0; border: 1px solid #3d3d3d; }
            QMenu::item:selected { background-color: #3d3d3d; }
            QTextEdit { background-color: #2a2a2a; color: #e0e0e0; border: 1px solid #3d3d3d; }
            QPlainTextEdit { background-color: #2a2a2a; color: #e0e0e0; border: 1px solid #3d3d3d; }
            QStatusBar { background-color: #2d2d2d; color: #e0e0e0; border-top: 1px solid #3d3d3d; }
            QLineEdit { background-color: #2d2d2d; color: #e0e0e0; border: 1px solid #3d3d3d; }
            QPushButton { background-color: #3d3d3d; color: #e0e0e0; border: 1px solid #4d4d4d; border-radius: 4px; padding: 5px; }
            QPushButton:hover { background-color: #4d4d4d; }
            QPushButton:pressed { background-color: #5d5d5d; }
        """
        self.setStyleSheet(dark_stylesheet)

    def _open_search(self):
        """Open the search/find dialog."""
        if self.stacked_widget.currentIndex() == 0:
            self.file_compare.show_search_dialog()

    def _open_go_to_line(self):
        """Open the go to line dialog."""
        if self.stacked_widget.currentIndex() == 0:
            self.file_compare.show_go_to_line_dialog()

    def _has_unsaved_changes(self) -> bool:
        """Check if there are unsaved changes in the file comparison."""
        return self.file_compare.is_left_modified or self.file_compare.is_right_modified

    def _show_unsaved_changes_dialog(self, action_name: str) -> bool:
        """
        Show a dialog asking user if they want to save unsaved changes.
        Returns True if user wants to proceed, False if cancelled.
        """
        dialog = QMessageBox(self)
        dialog.setWindowTitle("Unsaved Changes")
        dialog.setText("You have unsaved changes in the file comparison.")
        dialog.setInformativeText(f"Do you want to save before {action_name.lower()}?")
        dialog.setIcon(QMessageBox.Warning)

        # Add buttons
        save_button = dialog.addButton("Save", QMessageBox.AcceptRole)
        discard_button = dialog.addButton("Discard", QMessageBox.DestructiveRole)
        cancel_button = dialog.addButton("Cancel", QMessageBox.RejectRole)

        dialog.setDefaultButton(save_button)
        dialog.exec()

        clicked_button = dialog.clickedButton()
        if clicked_button == save_button:
            self.file_compare.save_left()
            self.file_compare.save_right()
            return True
        elif clicked_button == discard_button:
            return True
        else:  # Cancel
            return False

    def _show_about(self):
        """Show the About dialog."""
        about_text = """
        <h2>SmartMerge</h2>
        <p><b>A powerful file and folder comparison tool</b></p>
        <p>SmartMerge provides intelligent side-by-side comparison of files and folders 
        with advanced diff algorithms and an intuitive user interface.</p>
        <p><b>Features:</b></p>
        <ul>
            <li>File and folder comparison</li>
            <li>Multiple diff algorithms (Myers and Smart Block Diff)</li>
            <li>Light and Dark theme support</li>
            <li>Advanced search and navigation</li>
            <li>Undo/Redo support</li>
            <li>Syntax highlighting and line numbers</li>
            <li>Save and export capabilities</li>
        </ul>
        <p><b>Keyboard Shortcuts:</b></p>
        <ul>
            <li><code>Cmd+O</code> - Open Files</li>
            <li><code>Cmd+Shift+O</code> - Open Folders</li>
            <li><code>Alt+Down/Up</code> - Navigate changes</li>
            <li><code>Cmd+F</code> - Find</li>
            <li><code>Cmd+S</code> - Save</li>
        </ul>
        <p style="margin-top: 20px; font-size: 11px; color: gray;">
        SmartMerge - Making code review easier
        </p>
        """
        QMessageBox.about(self, "About SmartMerge", about_text)

    def closeEvent(self, event):
        # Save window geometry
        self.settings.setValue("window_geometry", self.saveGeometry())
        """Handle window close event."""
        if self._has_unsaved_changes():
            if not self._show_unsaved_changes_dialog("Quit"):
                event.ignore()
                return
        event.accept()


def main():
    app = QApplication(sys.argv)
    window = MainWindow()
    window.show()
    sys.exit(app.exec())


if __name__ == "__main__":
    main()
