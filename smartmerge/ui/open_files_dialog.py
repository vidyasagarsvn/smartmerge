
from PySide6.QtWidgets import QDialog, QVBoxLayout, QHBoxLayout, QPushButton, QLabel, QLineEdit, QFileDialog, QSizePolicy

class OpenFilesDialog(QDialog):
    """Dialog for selecting left and right files for comparison."""
    def __init__(self, parent=None):
        super().__init__(parent)
        self.setWindowTitle("Open Files")
        self.setModal(True)
        self.setFixedSize(500, 220)
        self.left_path = ''
        self.right_path = ''
        self.init_ui()

    def init_ui(self):
        layout = QVBoxLayout(self)

        # Left file row
        left_row = QHBoxLayout()
        left_label = QLabel("Left File:")
        self.left_edit = QLineEdit()
        self.left_edit.setReadOnly(True)
        self.left_edit.setDisabled(True)
        self.left_edit.setMinimumWidth(300)
        left_btn = QPushButton("Browse...")
        left_btn.setFixedWidth(90)
        left_btn.clicked.connect(self.browse_left)
        left_row.addWidget(left_label)
        left_row.addWidget(self.left_edit)
        left_row.addWidget(left_btn)

        # Right file row
        right_row = QHBoxLayout()
        right_label = QLabel("Right File:")
        self.right_edit = QLineEdit()
        self.right_edit.setReadOnly(True)
        self.right_edit.setDisabled(True)
        self.right_edit.setMinimumWidth(300)
        right_btn = QPushButton("Browse...")
        right_btn.setFixedWidth(90)
        right_btn.clicked.connect(self.browse_right)
        right_row.addWidget(right_label)
        right_row.addWidget(self.right_edit)
        right_row.addWidget(right_btn)

        # Action buttons
        btn_row = QHBoxLayout()
        open_btn = QPushButton("Open")
        cancel_btn = QPushButton("Cancel")
        open_btn.clicked.connect(self.accept)
        cancel_btn.clicked.connect(self.reject)
        btn_row.addStretch()
        btn_row.addWidget(open_btn)
        btn_row.addWidget(cancel_btn)

        layout.addLayout(left_row)
        layout.addSpacing(20)
        layout.addLayout(right_row)
        layout.addSpacing(20)
        layout.addLayout(btn_row)
        self.setLayout(layout)

    def browse_left(self):
        file_path, _ = QFileDialog.getOpenFileName(self, "Select Left File")
        if file_path:
            self.left_path = file_path
            self.left_edit.setText(file_path)

    def browse_right(self):
        file_path, _ = QFileDialog.getOpenFileName(self, "Select Right File")
        if file_path:
            self.right_path = file_path
            self.right_edit.setText(file_path)

    def get_paths(self):
        return self.left_path, self.right_path
