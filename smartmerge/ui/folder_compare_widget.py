"""Folder comparison UI widget."""

from pathlib import Path
from PySide6.QtWidgets import (
    QWidget,
    QTableWidget,
    QTableWidgetItem,
    QHeaderView,
    QMenu,
)
from PySide6.QtCore import Qt, Signal
from PySide6.QtGui import QColor, QFont

from smartmerge.core.folder_compare import (
    compare_folders,
    FolderItem,
    ItemType,
    ItemStatus,
    is_folder_navigable,
)


class FolderCompareWidget(QWidget):
    """Widget for displaying folder comparison results."""

    # Signal emitted when user wants to navigate into a folder
    folder_selected = Signal(Path, Path)

    # Signal emitted when user wants to compare a file
    file_selected = Signal(Path, Path)

    def __init__(self):
        super().__init__()
        self.left_path: Path | None = None
        self.right_path: Path | None = None
        self.current_items: list[FolderItem] = []
        self.initial_left_path: Path | None = (
            None  # Track initial paths for parent navigation
        )
        self.initial_right_path: Path | None = None
        self.theme: str = "light"

        self._setup_ui()

    def _setup_ui(self):
        """Initialize UI components."""
        from PySide6.QtWidgets import QVBoxLayout

        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)

        # Create table widget
        self.table = QTableWidget()
        self.table.setColumnCount(2)
        self.table.setHorizontalHeaderLabels(["Name", "Status"])

        # Configure columns
        header = self.table.horizontalHeader()
        header.setStretchLastSection(False)
        header.setSectionResizeMode(0, QHeaderView.Stretch)
        header.setSectionResizeMode(1, QHeaderView.ResizeToContents)

        # Set row height
        self.table.verticalHeader().setDefaultSectionSize(22)

        # Connect signals
        self.table.cellDoubleClicked.connect(self._on_cell_double_clicked)
        self.table.setContextMenuPolicy(Qt.CustomContextMenu)
        self.table.customContextMenuRequested.connect(self._on_context_menu)

        layout.addWidget(self.table)
        self._apply_theme_to_ui()

    def set_paths(self, left_path: Path, right_path: Path):
        """Set folder paths and update comparison."""
        self.left_path = left_path
        self.right_path = right_path
        # Only set initial paths if not already set (for breadcrumb tracking)
        if self.initial_left_path is None:
            self.initial_left_path = left_path
        if self.initial_right_path is None:
            self.initial_right_path = right_path
        self._update_table()

    def _update_table(self):
        """Update table with current folder comparison."""
        if not self.left_path or not self.right_path:
            self.table.setRowCount(0)
            return

        self.current_items = []

        # Add parent directory option if not at root level
        # Use resolve() to get absolute paths for proper comparison
        try:
            left_resolved = self.left_path.resolve()
            right_resolved = self.right_path.resolve()
            left_parent = left_resolved.parent
            right_parent = right_resolved.parent
            # Only show parent if both paths can actually go up
            if left_parent != left_resolved and right_parent != right_resolved:
                parent_item = FolderItem(
                    name="..",
                    type=ItemType.FOLDER,
                    left_path=left_parent,
                    right_path=right_parent,
                    status=ItemStatus.IDENTICAL,
                )
                self.current_items.append(parent_item)
        except (ValueError, OSError):
            # Handle any path errors gracefully
            pass

        # Add actual comparison items
        self.current_items.extend(compare_folders(self.left_path, self.right_path))
        self.table.setRowCount(len(self.current_items))

        for row, item in enumerate(self.current_items):
            self._populate_row(row, item)

    def _populate_row(self, row: int, item: FolderItem):
        """Populate a table row with item data."""
        # Name column
        name_item = QTableWidgetItem(item.name)
        if item.type == ItemType.FOLDER:
            name_item.setText(f"📁 {item.name}")
        else:
            name_item.setText(f"📄 {item.name}")

        # Disable editing
        name_item.setFlags(name_item.flags() & ~Qt.ItemIsEditable)

        # Make non-navigable folders grayed out
        if item.type == ItemType.FOLDER and not is_folder_navigable(item):
            gray = (
                QColor(150, 150, 150)
                if self.theme == "light"
                else QColor(100, 100, 100)
            )
            name_item.setForeground(gray)

        self.table.setItem(row, 0, name_item)

        # Status column - show icon and text for status
        status_icon, status_text = self._status_to_icon(item.status)
        status_item = QTableWidgetItem(f"{status_icon} {status_text}")
        status_item.setFlags(status_item.flags() & ~Qt.ItemIsEditable)
        self.table.setItem(row, 1, status_item)

    def _status_to_icon(self, status: ItemStatus) -> tuple[str, str]:
        """Convert status enum to icon and human-readable text."""
        status_map = {
            ItemStatus.IDENTICAL: ("✅", "Identical"),
            ItemStatus.MODIFIED: ("📝", "Modified"),
            ItemStatus.ADDED_LEFT: ("➡️", "Left Only"),
            ItemStatus.ADDED_RIGHT: ("⬅️", "Right Only"),
            ItemStatus.DELETED_LEFT: ("❌", "Left Missing"),
            ItemStatus.DELETED_RIGHT: ("❌", "Right Missing"),
            ItemStatus.FOLDER_LEFT_ONLY: ("➡️", "Left Only"),
            ItemStatus.FOLDER_RIGHT_ONLY: ("⬅️", "Right Only"),
        }
        icon, text = status_map.get(status, ("❓", "Unknown"))
        return icon, text

    def _on_cell_double_clicked(self, row: int, col: int):
        """Handle double-click on table cell."""
        if 0 <= row < len(self.current_items):
            item = self.current_items[row]

            # Only allow navigation into folders present on both sides
            if item.type == ItemType.FOLDER and is_folder_navigable(item):
                self.folder_selected.emit(item.left_path, item.right_path)

            # Allow comparison of files
            elif item.type == ItemType.FILE and item.left_path and item.right_path:
                self.file_selected.emit(item.left_path, item.right_path)

    def _on_context_menu(self, pos):
        """Show context menu for table."""
        item = self.table.itemAt(pos)
        if not item:
            return

        row = self.table.row(item)
        if 0 <= row < len(self.current_items):
            folder_item = self.current_items[row]

            menu = QMenu()

            if folder_item.type == ItemType.FILE:
                menu.addAction("Compare Files").triggered.connect(
                    lambda: self.file_selected.emit(
                        folder_item.left_path, folder_item.right_path
                    )
                )

            if folder_item.type == ItemType.FOLDER and is_folder_navigable(folder_item):
                menu.addAction("Navigate Into Folder").triggered.connect(
                    lambda: self.folder_selected.emit(
                        folder_item.left_path, folder_item.right_path
                    )
                )

            if menu.actions():
                menu.exec(self.table.mapToGlobal(pos))

    def set_font(self, font: QFont):
        """Update font for table display."""
        self.table.setFont(font)

    def set_theme(self, theme: str):
        """Set the current theme (light or dark) and update colors."""
        self.theme = theme
        self._apply_theme_to_ui()

    def _apply_theme_to_ui(self):
        """Apply theme styling to table and refresh display."""
        if self.theme == "dark":
            # Dark mode styling
            widget_stylesheet = "QWidget { background-color: #2a2a2a; }"
            table_stylesheet = """
                QTableWidget {
                    background-color: #2a2a2a;
                    color: #ffffff;
                    gridline-color: #3a3a3a;
                }
                QTableWidget::item {
                    padding: 2px;
                    border: none;
                }
                QHeaderView::section {
                    background-color: #3a3a3a;
                    color: #ffffff;
                    padding: 4px;
                    border: none;
                    border-right: 1px solid #2a2a2a;
                    border-bottom: 1px solid #2a2a2a;
                }
                QTableCornerButton::section {
                    background-color: #3a3a3a;
                }
                QScrollBar:vertical {
                    background-color: #2a2a2a;
                    width: 12px;
                }
                QScrollBar::handle:vertical {
                    background-color: #555555;
                    border-radius: 6px;
                }
                QScrollBar::handle:vertical:hover {
                    background-color: #666666;
                }
                QScrollBar:horizontal {
                    background-color: #2a2a2a;
                    height: 12px;
                }
                QScrollBar::handle:horizontal {
                    background-color: #555555;
                    border-radius: 6px;
                }
                QScrollBar::handle:horizontal:hover {
                    background-color: #666666;
                }
            """
        else:
            # Light mode styling
            widget_stylesheet = "QWidget { background-color: #ffffff; }"
            table_stylesheet = """
                QTableWidget {
                    background-color: #ffffff;
                    color: #000000;
                    gridline-color: #e0e0e0;
                }
                QTableWidget::item {
                    padding: 2px;
                    border: none;
                }
                QHeaderView::section {
                    background-color: #f5f5f5;
                    color: #000000;
                    padding: 4px;
                    border: none;
                    border-right: 1px solid #e0e0e0;
                    border-bottom: 1px solid #e0e0e0;
                }
                QTableCornerButton::section {
                    background-color: #f5f5f5;
                }
                QScrollBar:vertical {
                    background-color: #ffffff;
                    width: 12px;
                }
                QScrollBar::handle:vertical {
                    background-color: #cccccc;
                    border-radius: 6px;
                }
                QScrollBar::handle:vertical:hover {
                    background-color: #bbbbbb;
                }
                QScrollBar:horizontal {
                    background-color: #ffffff;
                    height: 12px;
                }
                QScrollBar::handle:horizontal {
                    background-color: #cccccc;
                    border-radius: 6px;
                }
                QScrollBar::handle:horizontal:hover {
                    background-color: #bbbbbb;
                }
            """

        self.setStyleSheet(widget_stylesheet)
        self.table.setStyleSheet(table_stylesheet)

        # Refresh table display to apply new colors
        if self.left_path and self.right_path:
            self._update_table()
