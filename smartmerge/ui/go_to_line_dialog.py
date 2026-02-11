"""Go to Line dialog for quick navigation."""

from PySide6.QtWidgets import (
    QDialog,
    QVBoxLayout,
    QHBoxLayout,
    QLineEdit,
    QPushButton,
    QLabel,
)
from PySide6.QtCore import Qt
from PySide6.QtGui import QIntValidator


class GoToLineDialog(QDialog):
    """Dialog for jumping to a specific line number."""

    def __init__(self, parent=None, max_lines: int = 100, theme: str = "light"):
        super().__init__(parent)
        self.theme = theme
        self.setWindowTitle("Go to Line")
        self.setModal(True)
        self.setFixedSize(300, 120)
        self.line_number = -1
        self.max_lines = max_lines
        self.init_ui()
        self._apply_theme()

    def init_ui(self):
        layout = QVBoxLayout(self)
        layout.setContentsMargins(15, 15, 15, 15)
        layout.setSpacing(10)

        # Line number input row
        input_row = QHBoxLayout()
        label = QLabel("Go to line:")
        label.setFixedWidth(80)

        self.line_input = QLineEdit()
        self.line_input.setPlaceholderText(f"1 - {self.max_lines}")
        self.line_input.setValidator(QIntValidator(1, self.max_lines))
        self.line_input.returnPressed.connect(self.accept)
        self.line_input.setFocus()

        input_row.addWidget(label)
        input_row.addWidget(self.line_input)

        # Buttons row
        buttons_row = QHBoxLayout()
        ok_btn = QPushButton("Go")
        ok_btn.setFixedWidth(80)
        ok_btn.clicked.connect(self.accept)

        cancel_btn = QPushButton("Cancel")
        cancel_btn.setFixedWidth(80)
        cancel_btn.clicked.connect(self.reject)

        buttons_row.addStretch()
        buttons_row.addWidget(ok_btn)
        buttons_row.addWidget(cancel_btn)

        layout.addLayout(input_row)
        layout.addLayout(buttons_row)
        self.setLayout(layout)

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

    def get_line_number(self) -> int:
        """Get the entered line number (1-indexed)."""
        try:
            return int(self.line_input.text())
        except ValueError:
            return -1
