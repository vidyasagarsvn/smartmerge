import sys
from PySide6.QtWidgets import QApplication, QMainWindow, QMenuBar, QStatusBar
from PySide6.QtGui import QAction
from PySide6.QtCore import Qt
from smartmerge.ui.file_compare_widget import FileCompareWidget
from smartmerge.ui.open_files_dialog import OpenFilesDialog

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
        from PySide6.QtWidgets import QDialog
        if dialog.exec() == QDialog.Accepted:
            left_path, right_path = dialog.get_paths()
            if left_path:
                self.compare_widget.load_left_file(left_path)
                self.statusBar.showMessage(f"Loaded left file: {left_path}")
            if right_path:
                self.compare_widget.load_right_file(right_path)
                self.statusBar.showMessage(f"Loaded right file: {right_path}")

def main():
    app = QApplication(sys.argv)
    window = MainWindow()
    window.show()
    sys.exit(app.exec())

if __name__ == "__main__":
    main()
