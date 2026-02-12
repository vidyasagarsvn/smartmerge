from PySide6.QtWidgets import (
    QDialog,
    QVBoxLayout,
    QHBoxLayout,
    QPushButton,
    QLabel,
    QLineEdit,
    QFileDialog,
)
from PySide6.QtCore import Qt
from PySide6.QtGui import QColor, QFont
from pathlib import Path


class OpenFilesDialog(QDialog):
    """Dialog for selecting left and right files for comparison."""

    def __init__(
        self, parent=None, initial_left_path="", initial_right_path="", theme="light"
    ):
        super().__init__(parent)
        self.theme = theme
        self.setWindowTitle("Open Files for Comparison")
        self.setModal(True)
        self.setFixedSize(600, 300)
        self.left_path = initial_left_path or ""
        self.right_path = initial_right_path or ""
        self.init_ui()
        self._apply_theme()

    def init_ui(self):
        main_layout = QVBoxLayout(self)
        main_layout.setContentsMargins(15, 15, 15, 15)
        main_layout.setSpacing(15)

        # Left file section
        left_label = QLabel("Left File:")
        left_input_row = QHBoxLayout()
        self.left_edit = QLineEdit()
        self.left_edit.setReadOnly(True)
        self.left_edit.setMinimumHeight(32)
        self.left_edit.setPlaceholderText("No file selected")
        if self.left_path:
            self.left_edit.setText(self.left_path)
        left_browse_btn = QPushButton("Browse...")
        left_browse_btn.setFixedWidth(100)
        left_browse_btn.setMinimumHeight(32)
        left_browse_btn.clicked.connect(self.browse_left)
        left_input_row.addWidget(self.left_edit)
        left_input_row.addWidget(left_browse_btn)

        # Left file status indicator
        self.left_status_label = QLabel()
        self.left_status_label.setWordWrap(True)

        # Right file section
        right_label = QLabel("Right File:")
        right_input_row = QHBoxLayout()
        self.right_edit = QLineEdit()
        self.right_edit.setReadOnly(True)
        self.right_edit.setMinimumHeight(32)
        self.right_edit.setPlaceholderText("No file selected")
        if self.right_path:
            self.right_edit.setText(self.right_path)
        right_browse_btn = QPushButton("Browse...")
        right_browse_btn.setFixedWidth(100)
        right_browse_btn.setMinimumHeight(32)
        right_browse_btn.clicked.connect(self.browse_right)
        right_input_row.addWidget(self.right_edit)
        right_input_row.addWidget(right_browse_btn)

        # Right file status indicator
        self.right_status_label = QLabel()
        self.right_status_label.setWordWrap(True)

        # Add all sections to main layout
        main_layout.addWidget(left_label)
        main_layout.addLayout(left_input_row)
        main_layout.addWidget(self.left_status_label)

        main_layout.addWidget(right_label)
        main_layout.addLayout(right_input_row)
        main_layout.addWidget(self.right_status_label)

        main_layout.addStretch()

        # Action buttons
        btn_row = QHBoxLayout()
        btn_row.setSpacing(10)

        self.cancel_btn = QPushButton("Cancel")
        self.cancel_btn.setFixedWidth(100)
        self.cancel_btn.setMinimumHeight(32)
        self.cancel_btn.clicked.connect(self.reject)

        self.open_btn = QPushButton("→ Compare")
        self.open_btn.setFixedWidth(100)
        self.open_btn.setMinimumHeight(32)
        self.open_btn.clicked.connect(self.accept)
        self.open_btn.setEnabled(False)

        btn_row.addStretch()
        btn_row.addWidget(self.cancel_btn)
        btn_row.addWidget(self.open_btn)
        main_layout.addLayout(btn_row)

        self.setLayout(main_layout)

        # Update status labels and button state based on initial paths
        if self.left_path:
            self._update_left_status()
        if self.right_path:
            self._update_right_status()
        self._update_button_state()

    def _apply_theme(self):
        """Apply the selected theme to the dialog."""
        if self.theme == "dark":
            dark_stylesheet = """
                QDialog { background-color: #2d2d2d; color: #e0e0e0; }
                QLineEdit { background-color: #3d3d3d; color: #e0e0e0; border: 1px solid #4d4d4d; padding: 5px; }
                QPushButton { background-color: #3d3d3d; color: #e0e0e0; border: 1px solid #4d4d4d; border-radius: 4px; padding: 5px; }
                QPushButton:hover { background-color: #4d4d4d; }
                QPushButton:pressed { background-color: #5d5d5d; }
                QLabel { color: #e0e0e0; }
            """
            self.setStyleSheet(dark_stylesheet)
        else:
            light_stylesheet = """
                QDialog { background-color: #ffffff; color: #000000; }
                QLineEdit { background-color: #ffffff; color: #000000; border: 1px solid #d0d0d0; padding: 5px; }
                QPushButton { background-color: #f0f0f0; color: #000000; border: 1px solid #d0d0d0; border-radius: 4px; padding: 5px; }
                QPushButton:hover { background-color: #e0e0e0; }
                QPushButton:pressed { background-color: #d0d0d0; }
                QLabel { color: #000000; }
            """
            self.setStyleSheet(light_stylesheet)

    def set_theme(self, theme: str):
        """Set the current theme and update styling."""
        self.theme = theme
        self._apply_theme()

        # Initialize with existing paths if provided
        if self.left_path:
            self.left_edit.setText(self.left_path)
            self._update_left_status()
        if self.right_path:
            self.right_edit.setText(self.right_path)
            self._update_right_status()

        self._update_button_state()

    def browse_left(self):
        file_path, _ = QFileDialog.getOpenFileName(self, "Select Left File")
        if file_path:
            self.left_path = file_path
            self.left_edit.setText(file_path)
            self._update_left_status()
            self._update_button_state()

    def browse_right(self):
        file_path, _ = QFileDialog.getOpenFileName(self, "Select Right File")
        if file_path:
            self.right_path = file_path
            self.right_edit.setText(file_path)
            self._update_right_status()
            self._update_button_state()

    def _update_left_status(self) -> None:
        """Update the left file status indicator."""
        if not self.left_path:
            self.left_status_label.setText("ⓘ Select a file to compare")
            self.left_status_label.setStyleSheet("color: #666; font-size: 11px;")
            return

        path = Path(self.left_path)
        if path.exists():
            size = path.stat().st_size
            size_str = self._format_file_size(size)
            self.left_status_label.setText(f"✓ File exists • {size_str}")
            self.left_status_label.setStyleSheet(
                "color: #4CAF50; font-size: 11px; font-weight: bold;"
            )
        else:
            self.left_status_label.setText("✗ File not found")
            self.left_status_label.setStyleSheet(
                "color: #f44336; font-size: 11px; font-weight: bold;"
            )

    def _update_right_status(self) -> None:
        """Update the right file status indicator."""
        if not self.right_path:
            self.right_status_label.setText("ⓘ Select a file to compare")
            self.right_status_label.setStyleSheet("color: #666; font-size: 11px;")
            return

        path = Path(self.right_path)
        if path.exists():
            size = path.stat().st_size
            size_str = self._format_file_size(size)
            self.right_status_label.setText(f"✓ File exists • {size_str}")
            self.right_status_label.setStyleSheet(
                "color: #4CAF50; font-size: 11px; font-weight: bold;"
            )
        else:
            self.right_status_label.setText("✗ File not found")
            self.right_status_label.setStyleSheet(
                "color: #f44336; font-size: 11px; font-weight: bold;"
            )

    def _format_file_size(self, size: int) -> str:
        """Format file size in human-readable format."""
        for unit in ["B", "KB", "MB", "GB"]:
            if size < 1024:
                return f"{size:.1f} {unit}"
            size /= 1024
        return f"{size:.1f} TB"

    def _update_button_state(self) -> None:
        """Enable/disable Open button based on file selection."""
        left_exists = self.left_path and Path(self.left_path).exists()
        right_exists = self.right_path and Path(self.right_path).exists()
        self.open_btn.setEnabled(left_exists and right_exists)

    def get_paths(self):
        return self.left_path, self.right_path
