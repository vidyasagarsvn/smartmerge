import sys
from PySide6.QtWidgets import (
    QApplication, QMainWindow, QWidget, QHBoxLayout, QVBoxLayout,
    QTextEdit, QPushButton, QFileDialog, QLabel, QSplitter, QMenuBar, QStatusBar, QDialog, QLineEdit
)
class OpenFilesDialog(QDialog):
    def __init__(self, parent=None):
        super().__init__(parent)
        self.setWindowTitle("Open Files")
        self.setModal(True)

        # === Imports ===
        import sys
        from PySide6.QtWidgets import (
            QApplication, QMainWindow, QWidget, QHBoxLayout, QVBoxLayout,
            QTextEdit, QPushButton, QFileDialog, QLabel, QSplitter, QMenuBar, QStatusBar, QDialog
        )
        from PySide6.QtGui import QAction
        from PySide6.QtCore import Qt


        # === Dialogs ===
        class OpenFilesDialog(QDialog):
            """Dialog for selecting left and right files for comparison."""
            def __init__(self, parent=None):
                super().__init__(parent)
                self.setWindowTitle("Open Files")
                self.setModal(True)
                self.left_path = ''
                self.right_path = ''
                self.init_ui()

            def init_ui(self):
                layout = QVBoxLayout(self)
                self.left_path = ''
                self.right_path = ''
                left_btn = QPushButton("Select Left File")
                right_btn = QPushButton("Select Right File")
                left_btn.setMinimumHeight(40)
                right_btn.setMinimumHeight(40)
                self.left_label = QLabel("No file selected")
                self.right_label = QLabel("No file selected")
                open_btn = QPushButton("Open")
                cancel_btn = QPushButton("Cancel")

                left_btn.clicked.connect(self.browse_left)
                right_btn.clicked.connect(self.browse_right)
                open_btn.clicked.connect(self.accept)
                cancel_btn.clicked.connect(self.reject)

                layout.addWidget(QLabel("Left File:"))
                layout.addWidget(left_btn)
                layout.addWidget(self.left_label)
                layout.addSpacing(10)
                layout.addWidget(QLabel("Right File:"))
                layout.addWidget(right_btn)
                layout.addWidget(self.right_label)
                layout.addSpacing(10)
                btn_layout = QHBoxLayout()
                btn_layout.addWidget(open_btn)
                btn_layout.addWidget(cancel_btn)
                layout.addLayout(btn_layout)
                self.setLayout(layout)

            def browse_left(self):
                file_path, _ = QFileDialog.getOpenFileName(self, "Select Left File")
                if file_path:
                    self.left_path = file_path
                    self.left_label.setText(file_path)

            def browse_right(self):
                file_path, _ = QFileDialog.getOpenFileName(self, "Select Right File")
                if file_path:
                    self.right_path = file_path
                    self.right_label.setText(file_path)

            def get_paths(self):
                return self.left_path, self.right_path


        # === Widgets ===
        class FileCompareWidget(QWidget):
            """Widget for displaying and comparing two files side by side."""
            def __init__(self, parent=None):
                super().__init__(parent)
                self.init_ui()

            def init_ui(self):
                layout = QHBoxLayout(self)
                self.left_text = QTextEdit()
                self.right_text = QTextEdit()
                self.left_text.setPlaceholderText("Open left file...")
                self.right_text.setPlaceholderText("Open right file...")
                splitter = QSplitter(Qt.Horizontal)
                splitter.addWidget(self.left_text)
                splitter.addWidget(self.right_text)
                layout.addWidget(splitter)
                self.setLayout(layout)

            def load_left_file(self, left_path):
                with open(left_path, 'r', encoding='utf-8', errors='ignore') as f:
                    self.left_text.setPlainText(f.read())

            def load_right_file(self, right_path):
                with open(right_path, 'r', encoding='utf-8', errors='ignore') as f:
                    self.right_text.setPlainText(f.read())


        # === Main Window ===
        self.right_path = ''
        left_btn = QPushButton("Select Left File")
        right_btn = QPushButton("Select Right File")
        left_btn.setMinimumHeight(40)
        right_btn.setMinimumHeight(40)
        self.left_label = QLabel("No file selected")
        self.right_label = QLabel("No file selected")
        open_btn = QPushButton("Open")
        cancel_btn = QPushButton("Cancel")

        left_btn.clicked.connect(self.browse_left)
        right_btn.clicked.connect(self.browse_right)
        open_btn.clicked.connect(self.accept)
        cancel_btn.clicked.connect(self.reject)

        layout.addWidget(QLabel("Left File:"))
        layout.addWidget(left_btn)
        layout.addWidget(self.left_label)
        layout.addSpacing(10)
        layout.addWidget(QLabel("Right File:"))
        layout.addWidget(right_btn)
        layout.addWidget(self.right_label)
        layout.addSpacing(10)
        btn_layout = QHBoxLayout()
        btn_layout.addWidget(open_btn)
        btn_layout.addWidget(cancel_btn)
        layout.addLayout(btn_layout)
        self.setLayout(layout)

    def browse_left(self):
        file_path, _ = QFileDialog.getOpenFileName(self, "Select Left File")
        if file_path:
            self.left_path = file_path
            self.left_label.setText(file_path)

        # === Main Entry Point ===
        def main():
            app = QApplication(sys.argv)
            window = MainWindow()
            window.show()
            sys.exit(app.exec())

        if __name__ == "__main__":
            main()
from PySide6.QtGui import QAction
from PySide6.QtCore import Qt

class FileCompareWidget(QWidget):
    def __init__(self, parent=None):
        super().__init__(parent)
        self.init_ui()

    def init_ui(self):
        layout = QHBoxLayout(self)
        self.left_text = QTextEdit()
        self.right_text = QTextEdit()
        self.left_text.setPlaceholderText("Open left file...")
        self.right_text.setPlaceholderText("Open right file...")
        splitter = QSplitter(Qt.Horizontal)
        splitter.addWidget(self.left_text)
        splitter.addWidget(self.right_text)
        layout.addWidget(splitter)
        self.setLayout(layout)

    def load_left_file(self, left_path):
        with open(left_path, 'r', encoding='utf-8', errors='ignore') as f:
            self.left_text.setPlainText(f.read())

    def load_right_file(self, right_path):
        with open(right_path, 'r', encoding='utf-8', errors='ignore') as f:
            self.right_text.setPlainText(f.read())

class MainWindow(QMainWindow):
    def __init__(self):
        super().__init__()
        self.setWindowTitle("SmartMerge - File Compare")
        self.resize(1200, 700)
        self.compare_widget = FileCompareWidget()
        self.setCentralWidget(self.compare_widget)
        self._create_menu()
        self.statusBar = QStatusBar()
        self.setStatusBar(self.statusBar)

    def _create_menu(self):
        menubar = QMenuBar(self)
        file_menu = menubar.addMenu("File")
        open_files_action = QAction("Open Files", self)
        exit_action = QAction("Exit", self)
        open_files_action.triggered.connect(self.open_files_dialog)
        exit_action.triggered.connect(self.close)
        file_menu.addAction(open_files_action)
        file_menu.addSeparator()
        file_menu.addAction(exit_action)
        self.setMenuBar(menubar)

    def open_files_dialog(self):
        dialog = OpenFilesDialog(self)
        if dialog.exec() == QDialog.Accepted:
            left_path, right_path = dialog.get_paths()
            if left_path:
                self.compare_widget.load_left_file(left_path)
                self.statusBar.showMessage(f"Loaded left file: {left_path}")
            if right_path:
                self.compare_widget.load_right_file(right_path)
                self.statusBar.showMessage(f"Loaded right file: {right_path}")

    # Removed open_left_file and open_right_file methods

def main():
    app = QApplication(sys.argv)
    window = MainWindow()
    window.show()
    sys.exit(app.exec())

if __name__ == "__main__":
    main()
