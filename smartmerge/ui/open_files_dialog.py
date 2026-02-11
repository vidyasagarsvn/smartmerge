from PySide6.QtWidgets import (
    QDialog,
    QVBoxLayout,
    QHBoxLayout,
    QPushButton,
    QLabel,
    QLineEdit,
    QFileDialog,
    QGroupBox,
)
from PySide6.QtCore import Qt
from PySide6.QtGui import QColor, QFont
from pathlib import Path


class OpenFilesDialog(QDialog):
    """Dialog for selecting left and right files for comparison."""

    def __init__(self, parent=None, initial_left_path="", initial_right_path=""):
        super().__init__(parent)
        self.setWindowTitle("Open Files for Comparison")
        self.setModal(True)
        self.setFixedSize(650, 380)
        self.left_path = initial_left_path or ""
        self.right_path = initial_right_path or ""
        self.init_ui()

    def init_ui(self):
        main_layout = QVBoxLayout(self)
        main_layout.setSpacing(15)
        main_layout.setContentsMargins(20, 20, 20, 20)

        # Title
        title = QLabel("Select files to compare:")
        title_font = QFont()
        title_font.setPointSize(12)
        title_font.setBold(True)
        title.setFont(title_font)
        main_layout.addWidget(title)

        # Left file section with group box
        left_group = QGroupBox("← Left File", self)
        left_group.setStyleSheet("""
            QGroupBox {
                color: #333;
                border: 1px solid #ddd;
                border-radius: 6px;
                margin-top: 8px;
                padding-top: 12px;
            }
            QGroupBox::title {
                subcontrol-origin: margin;
                left: 10px;
                padding: 0 3px 0 3px;
                font-weight: bold;
            }
        """)
        left_layout = QVBoxLayout(left_group)
        left_layout.setContentsMargins(15, 10, 15, 15)
        left_layout.setSpacing(8)

        # Left file input row
        left_input_row = QHBoxLayout()
        self.left_edit = QLineEdit()
        self.left_edit.setReadOnly(True)
        self.left_edit.setMinimumHeight(32)
        self.left_edit.setPlaceholderText("No file selected")
        left_browse_btn = QPushButton("Browse...")
        left_browse_btn.setFixedWidth(100)
        left_browse_btn.setMinimumHeight(32)
        left_browse_btn.clicked.connect(self.browse_left)
        left_input_row.addWidget(self.left_edit)
        left_input_row.addWidget(left_browse_btn)
        left_layout.addLayout(left_input_row)

        # Left file status indicator
        self.left_status_label = QLabel("ⓘ Select a file to compare")
        self.left_status_label.setStyleSheet(
            "color: #666; font-size: 10px; margin-top: 4px;"
        )
        self.left_status_label.setWordWrap(True)
        left_layout.addWidget(self.left_status_label)

        # Right file section with group box
        right_group = QGroupBox("→ Right File", self)
        right_group.setStyleSheet("""
            QGroupBox {
                color: #333;
                border: 1px solid #ddd;
                border-radius: 6px;
                margin-top: 8px;
                padding-top: 12px;
            }
            QGroupBox::title {
                subcontrol-origin: margin;
                left: 10px;
                padding: 0 3px 0 3px;
                font-weight: bold;
            }
        """)
        right_layout = QVBoxLayout(right_group)
        right_layout.setContentsMargins(15, 10, 15, 15)
        right_layout.setSpacing(8)

        # Right file input row
        right_input_row = QHBoxLayout()
        self.right_edit = QLineEdit()
        self.right_edit.setReadOnly(True)
        self.right_edit.setMinimumHeight(32)
        self.right_edit.setPlaceholderText("No file selected")
        right_browse_btn = QPushButton("Browse...")
        right_browse_btn.setFixedWidth(100)
        right_browse_btn.setMinimumHeight(32)
        right_browse_btn.clicked.connect(self.browse_right)
        right_input_row.addWidget(self.right_edit)
        right_input_row.addWidget(right_browse_btn)
        right_layout.addLayout(right_input_row)

        # Right file status indicator
        self.right_status_label = QLabel("ⓘ Select a file to compare")
        self.right_status_label.setStyleSheet(
            "color: #666; font-size: 10px; margin-top: 4px;"
        )
        self.right_status_label.setWordWrap(True)
        right_layout.addWidget(self.right_status_label)

        main_layout.addWidget(left_group)
        main_layout.addWidget(right_group)
        main_layout.addStretch()

        # Action buttons
        btn_row = QHBoxLayout()
        self.cancel_btn = QPushButton("Cancel")
        self.cancel_btn.setFixedWidth(100)
        self.cancel_btn.setMinimumHeight(36)
        self.cancel_btn.clicked.connect(self.reject)

        self.open_btn = QPushButton("→ Compare Files")
        self.open_btn.setFixedWidth(140)
        self.open_btn.setMinimumHeight(36)
        self.open_btn.setStyleSheet("""
            QPushButton {
                background-color: #4CAF50;
                color: white;
                border: none;
                border-radius: 4px;
                font-weight: bold;
            }
            QPushButton:hover:!pressed {
                background-color: #45a049;
            }
            QPushButton:pressed {
                background-color: #3d8b40;
            }
            QPushButton:disabled {
                background-color: #cccccc;
                color: #666;
            }
        """)
        self.open_btn.clicked.connect(self.accept)
        self.open_btn.setEnabled(False)

        btn_row.addStretch()
        btn_row.addWidget(self.cancel_btn)
        btn_row.addWidget(self.open_btn)
        main_layout.addLayout(btn_row)

        self.setLayout(main_layout)

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
            self.left_status_label.setStyleSheet("color: #666; font-size: 10px;")
            return

        path = Path(self.left_path)
        if path.exists():
            size = path.stat().st_size
            size_str = self._format_file_size(size)
            self.left_status_label.setText(f"✓ File exists • {size_str}")
            self.left_status_label.setStyleSheet(
                "color: #4CAF50; font-size: 10px; font-weight: bold;"
            )
        else:
            self.left_status_label.setText("✗ File not found")
            self.left_status_label.setStyleSheet(
                "color: #f44336; font-size: 10px; font-weight: bold;"
            )

    def _update_right_status(self) -> None:
        """Update the right file status indicator."""
        if not self.right_path:
            self.right_status_label.setText("ⓘ Select a file to compare")
            self.right_status_label.setStyleSheet("color: #666; font-size: 10px;")
            return

        path = Path(self.right_path)
        if path.exists():
            size = path.stat().st_size
            size_str = self._format_file_size(size)
            self.right_status_label.setText(f"✓ File exists • {size_str}")
            self.right_status_label.setStyleSheet(
                "color: #4CAF50; font-size: 10px; font-weight: bold;"
            )
        else:
            self.right_status_label.setText("✗ File not found")
            self.right_status_label.setStyleSheet(
                "color: #f44336; font-size: 10px; font-weight: bold;"
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
