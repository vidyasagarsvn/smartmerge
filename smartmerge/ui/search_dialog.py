"""Search/Find dialog for comparing files."""

from PySide6.QtWidgets import (
    QDialog,
    QVBoxLayout,
    QHBoxLayout,
    QLineEdit,
    QPushButton,
    QLabel,
    QCheckBox,
)
from PySide6.QtCore import Qt, Signal
from PySide6.QtGui import QFont


class SearchDialog(QDialog):
    """Find/Search dialog with options."""

    search_requested = Signal(str, bool)  # (search_text, case_sensitive)
    find_next = Signal()
    find_prev = Signal()

    def __init__(self, parent=None, theme: str = "light"):
        super().__init__(parent)
        self.theme = theme
        self.setWindowTitle("Find")
        self.setModal(False)
        self.setFixedSize(550, 140)
        self.init_ui()
        self._apply_theme()

    def init_ui(self):
        layout = QVBoxLayout(self)
        layout.setContentsMargins(15, 15, 15, 15)
        layout.setSpacing(12)

        # Search input row
        search_row = QHBoxLayout()
        search_label = QLabel("Find:")
        search_label.setFixedWidth(50)
        self.search_input = QLineEdit()
        self.search_input.setPlaceholderText("Enter text to find...")
        self.search_input.setMinimumHeight(32)
        self.search_input.returnPressed.connect(self._on_search)
        search_row.addWidget(search_label)
        search_row.addWidget(self.search_input)

        # Buttons and options row
        buttons_row = QHBoxLayout()
        buttons_row.setSpacing(10)

        find_prev_btn = QPushButton("← Previous")
        find_prev_btn.setMinimumHeight(32)
        find_prev_btn.setFixedWidth(110)
        find_prev_btn.setEnabled(True)
        find_prev_btn.clicked.connect(self.find_prev.emit)

        find_next_btn = QPushButton("Next →")
        find_next_btn.setMinimumHeight(32)
        find_next_btn.setFixedWidth(110)
        find_next_btn.setEnabled(True)
        find_next_btn.clicked.connect(self.find_next.emit)

        # Case sensitive checkbox
        self.case_sensitive_check = QCheckBox("Case sensitive")
        self.case_sensitive_check.setChecked(False)

        close_btn = QPushButton("Close")
        close_btn.setMinimumHeight(32)
        close_btn.setFixedWidth(80)
        close_btn.clicked.connect(self.close)

        buttons_row.addWidget(find_prev_btn)
        buttons_row.addWidget(find_next_btn)
        buttons_row.addWidget(self.case_sensitive_check)
        buttons_row.addStretch()
        buttons_row.addWidget(close_btn)

        layout.addLayout(search_row)
        layout.addLayout(buttons_row)
        self.setLayout(layout)

        # Focus on search input
        self.search_input.setFocus()

    def _on_search(self):
        """Emit search signal when user presses Enter."""
        text = self.search_input.text()
        if text:
            self.search_requested.emit(text, self.case_sensitive_check.isChecked())
            self.find_next.emit()

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
                QCheckBox { color: #e0e0e0; }
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
                QCheckBox { color: #000000; }
            """
            self.setStyleSheet(light_stylesheet)

    def get_search_text(self) -> str:
        """Get the current search text."""
        return self.search_input.text()

    def is_case_sensitive(self) -> bool:
        """Check if case sensitive search is enabled."""
        return self.case_sensitive_check.isChecked()
