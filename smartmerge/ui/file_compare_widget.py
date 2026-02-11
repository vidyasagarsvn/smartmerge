from PySide6.QtWidgets import QWidget, QHBoxLayout, QTextEdit, QSplitter, QVBoxLayout
from PySide6.QtCore import Qt
from PySide6.QtGui import QTextCharFormat, QColor, QPalette
from smartmerge.core.diff_engine import myers_opcodes, normalize_opcodes
from smartmerge.core.smart_diff_engine import smart_diff


class FileCompareWidget(QWidget):
    """Widget for displaying and comparing two files side by side with line-level diff highlighting."""

    def __init__(self, parent=None):
        super().__init__(parent)
        self.init_ui()
        self.current_font = self.left_text.font()
        self.left_lines = []
        self.right_lines = []
        self.diff_engine = "smart"  # Default to Smart Block Diff engine

    def init_ui(self):
        layout = QHBoxLayout(self)
        # Left panel
        self.left_panel = QVBoxLayout()
        self.left_text = QTextEdit()
        self.left_text.setPlaceholderText("Open left file...")
        self.left_text.setReadOnly(True)
        self.left_text.setStyleSheet(
            "QTextEdit { background-color: #ffffff; color: #111827; "
            "selection-background-color: #f0f0f0; }"
        )
        self.left_text.setFocusPolicy(Qt.NoFocus)  # Disable focus highlighting
        self._apply_light_palette(self.left_text)
        self.left_panel.addWidget(self.left_text)
        # Right panel
        self.right_panel = QVBoxLayout()
        self.right_text = QTextEdit()
        self.right_text.setPlaceholderText("Open right file...")
        self.right_text.setReadOnly(True)
        self.right_text.setStyleSheet(
            "QTextEdit { background-color: #ffffff; color: #111827; "
            "selection-background-color: #f0f0f0; }"
        )
        self.right_text.setFocusPolicy(Qt.NoFocus)  # Disable focus highlighting
        self._apply_light_palette(self.right_text)
        self.right_panel.addWidget(self.right_text)
        # Merge controls panel
        self.controls_panel = QVBoxLayout()
        self.controls_panel.setAlignment(Qt.AlignTop)
        # Add panels to splitter
        splitter = QSplitter(Qt.Horizontal)
        left_widget = QWidget()
        left_widget.setLayout(self.left_panel)
        right_widget = QWidget()
        right_widget.setLayout(self.right_panel)
        controls_widget = QWidget()
        controls_widget.setLayout(self.controls_panel)
        controls_widget.setFixedWidth(40)
        splitter.addWidget(left_widget)
        splitter.addWidget(controls_widget)
        splitter.addWidget(right_widget)
        splitter.setStretchFactor(0, 1)
        splitter.setStretchFactor(1, 0)
        splitter.setStretchFactor(2, 1)
        layout.addWidget(splitter)
        self.setLayout(layout)

    def load_left_file(self, left_path):
        with open(left_path, "r", encoding="utf-8", errors="ignore") as f:
            self.left_lines = f.readlines()
        self._update_views()

    def load_right_file(self, right_path):
        with open(right_path, "r", encoding="utf-8", errors="ignore") as f:
            self.right_lines = f.readlines()
        self._update_views()

    def set_font(self, font):
        self.current_font = font
        self.left_text.setFont(font)
        self.right_text.setFont(font)
        self._update_views()

    def _update_views(self):
        if self.current_font:
            self.left_text.setFont(self.current_font)
            self.right_text.setFont(self.current_font)

        # Use the selected diff engine
        if self.diff_engine == "smart":
            opcodes = smart_diff(self.left_lines, self.right_lines)
        else:  # Default to Myers
            opcodes = myers_opcodes(self.left_lines, self.right_lines)
            opcodes = normalize_opcodes(
                opcodes, len(self.left_lines), len(self.right_lines)
            )
        # Line highlight formats
        change_format = QTextCharFormat()
        change_format.setBackground(QColor("#ffff99"))  # changed (yellow)
        insert_format = QTextCharFormat()
        insert_format.setBackground(QColor("#ccffcc"))  # added (green)
        delete_format = QTextCharFormat()
        delete_format.setBackground(QColor("#ffcccc"))  # deleted (red)
        # Clear previous merge controls
        for i in reversed(range(self.controls_panel.count())):
            widget = self.controls_panel.itemAt(i).widget()
            if widget:
                widget.setParent(None)
        left_lines_out = []
        right_lines_out = []
        left_formats = []
        right_formats = []
        region_map = []  # Store region diff info for each line (unused for full-line)
        # For each opcode, build output lines and region info
        for tag, i1, i2, j1, j2 in opcodes:
            if tag == "equal":
                for i, j in zip(range(i1, i2), range(j1, j2)):
                    left_lines_out.append(self.left_lines[i].rstrip("\n"))
                    right_lines_out.append(self.right_lines[j].rstrip("\n"))
                    left_formats.append(None)
                    right_formats.append(None)
                    region_map.append(None)
            elif tag == "replace":
                llen = i2 - i1
                rlen = j2 - j1
                maxlen = max(llen, rlen)
                for k in range(maxlen):
                    left_line = self.left_lines[i1 + k].rstrip("\n") if k < llen else ""
                    right_line = (
                        self.right_lines[j1 + k].rstrip("\n") if k < rlen else ""
                    )
                    left_lines_out.append(left_line)
                    right_lines_out.append(right_line)
                    region_map.append(None)
                    left_formats.append(change_format)
                    right_formats.append(change_format)
            elif tag == "insert":
                for k in range(j1, j2):
                    left_lines_out.append("")
                    left_formats.append(None)
                    right_lines_out.append(self.right_lines[k].rstrip("\n"))
                    right_formats.append(insert_format)
                    region_map.append(None)
            elif tag == "delete":
                for k in range(i1, i2):
                    left_lines_out.append(self.left_lines[k].rstrip("\n"))
                    left_formats.append(delete_format)
                    right_lines_out.append("")
                    right_formats.append(None)
                    region_map.append(None)

        # Output to text widgets
        self.left_text.clear()
        self.right_text.clear()
        if not left_lines_out and self.left_lines:
            self.left_text.setPlainText("".join(self.left_lines))
        if not right_lines_out and self.right_lines:
            self.right_text.setPlainText("".join(self.right_lines))
        if not left_lines_out and not right_lines_out:
            return
        for idx, (left_line_text, left_format) in enumerate(
            zip(left_lines_out, left_formats)
        ):
            if left_format:
                self._append_highlighted(self.left_text, left_line_text, left_format)
            else:
                self._append_plain(self.left_text, left_line_text)
        for idx, (right_line_text, right_format) in enumerate(
            zip(right_lines_out, right_formats)
        ):
            if right_format:
                self._append_highlighted(self.right_text, right_line_text, right_format)
            else:
                self._append_plain(self.right_text, right_line_text)

    def _apply_light_palette(self, text_edit):
        palette = text_edit.palette()
        palette.setColor(QPalette.Base, QColor("#ffffff"))
        palette.setColor(QPalette.Text, QColor("#111827"))
        text_edit.setPalette(palette)

    def _append_highlighted(self, text_edit, line, fmt):
        from PySide6.QtGui import QTextCursor
        from PySide6.QtGui import QTextBlockFormat

        cursor = text_edit.textCursor()
        cursor.movePosition(QTextCursor.MoveOperation.End)
        effective_format = QTextCharFormat(fmt)
        if self.current_font:
            effective_format.setFont(self.current_font)
        block_format = QTextBlockFormat()
        block_format.setBackground(fmt.background())
        cursor.setBlockFormat(block_format)
        cursor.insertText(line, effective_format)
        cursor.insertText("\n")
        text_edit.setTextCursor(cursor)

    def _append_plain(self, text_edit, line):
        from PySide6.QtGui import QTextCursor
        from PySide6.QtGui import QTextBlockFormat

        cursor = text_edit.textCursor()
        cursor.movePosition(QTextCursor.MoveOperation.End)
        block_format = QTextBlockFormat()
        block_format.clearBackground()
        cursor.setBlockFormat(block_format)
        cursor.insertText(line)
        cursor.insertText("\n")
        text_edit.setTextCursor(cursor)

    def set_diff_engine(self, engine_name: str):
        """Set the diff engine to use (myers or smart)."""
        if engine_name in ("myers", "smart"):
            self.diff_engine = engine_name
            # Re-render with the new engine
            self._update_views()
