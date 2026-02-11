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

        self._setup_ui()

    def _setup_ui(self):
        """Initialize UI components."""
        from PySide6.QtWidgets import QVBoxLayout

        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)

        # Create table widget
        self.table = QTableWidget()
        self.table.setColumnCount(4)
        self.table.setHorizontalHeaderLabels(["Name", "Type", "Left", "Right"])

        # Configure columns
        header = self.table.horizontalHeader()
        header.setStretchLastSection(False)
        header.setSectionResizeMode(0, QHeaderView.Stretch)
        header.setSectionResizeMode(1, QHeaderView.ResizeToContents)
        header.setSectionResizeMode(2, QHeaderView.ResizeToContents)
        header.setSectionResizeMode(3, QHeaderView.ResizeToContents)

        # Set row height
        self.table.verticalHeader().setDefaultSectionSize(22)

        # Connect signals
        self.table.cellDoubleClicked.connect(self._on_cell_double_clicked)
        self.table.setContextMenuPolicy(Qt.CustomContextMenu)
        self.table.customContextMenuRequested.connect(self._on_context_menu)

        layout.addWidget(self.table)

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
            gray = QColor(150, 150, 150)
            name_item.setForeground(gray)

        self.table.setItem(row, 0, name_item)

        # Type column
        type_item = QTableWidgetItem(item.type.value)
        type_item.setFlags(type_item.flags() & ~Qt.ItemIsEditable)
        self.table.setItem(row, 1, type_item)

        # Left status column
        left_text, left_color = self._status_to_display(item.status, side="left")
        left_item = QTableWidgetItem(left_text)
        left_item.setFlags(left_item.flags() & ~Qt.ItemIsEditable)
        if left_color:
            left_item.setBackground(left_color)
        self.table.setItem(row, 2, left_item)

        # Right status column
        right_text, right_color = self._status_to_display(item.status, side="right")
        right_item = QTableWidgetItem(right_text)
        right_item.setFlags(right_item.flags() & ~Qt.ItemIsEditable)
        if right_color:
            right_item.setBackground(right_color)
        self.table.setItem(row, 3, right_item)

    def _status_to_display(
        self, status: ItemStatus, side: str
    ) -> tuple[str, QColor | None]:
        """Convert status enum to display text and color."""
        if status == ItemStatus.IDENTICAL:
            return "✓", QColor(200, 255, 200)  # Light green
        elif status == ItemStatus.MODIFIED:
            return "≠", QColor(255, 255, 200)  # Light yellow
        elif status == ItemStatus.ADDED_LEFT:
            return "+" if side == "left" else "", QColor(
                200, 255, 200
            ) if side == "left" else None
        elif status == ItemStatus.ADDED_RIGHT:
            return "" if side == "left" else "+", QColor(
                200, 255, 200
            ) if side == "right" else None
        elif status == ItemStatus.DELETED_LEFT:
            return "-" if side == "left" else "", QColor(
                255, 200, 200
            ) if side == "left" else None
        elif status == ItemStatus.DELETED_RIGHT:
            return "" if side == "left" else "-", QColor(
                255, 200, 200
            ) if side == "right" else None
        elif status == ItemStatus.FOLDER_LEFT_ONLY:
            return "📁" if side == "left" else "", None
        elif status == ItemStatus.FOLDER_RIGHT_ONLY:
            return "" if side == "left" else "📁", None

        return "", None

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
