from PySide6.QtWidgets import (
    QWidget,
    QHBoxLayout,
    QTextEdit,
    QSplitter,
    QVBoxLayout,
    QLabel,
)
from PySide6.QtCore import Qt
from PySide6.QtGui import QTextCharFormat, QColor, QPalette
from smartmerge.core.diff_engine import myers_opcodes, normalize_opcodes
from smartmerge.core.smart_diff_engine import smart_diff
from smartmerge.ui.syntax_highlighter import SyntaxHighlighter


class FileCompareWidget(QWidget):
    """Widget for displaying and comparing two files side by side with line-level diff highlighting."""

    def __init__(self, parent=None):
        super().__init__(parent)
        self.init_ui()
        self.current_font = self.left_text.font()
        self.left_lines = []
        self.right_lines = []
        self.diff_engine = "smart"  # Default to Smart Block Diff engine
        self._is_syncing_scroll = False
        self._change_regions = []  # List of (start_line, end_line) tuples for each change region
        self._current_region_index = -1
        self._current_region_lines = None

    def init_ui(self):
        layout = QHBoxLayout(self)
        # Left panel
        self.left_panel = QVBoxLayout()
        self.left_file_label = QLabel("No file selected")
        self.left_file_label.setStyleSheet(
            "QLabel { background-color: #f3f4f6; color: #374151; padding: 8px 12px; "
            "border-bottom: 1px solid #d1d5db; font-weight: 500; font-size: 12px; }"
        )
        self.left_panel.addWidget(self.left_file_label)
        self.left_text = QTextEdit()
        self.left_text.setPlaceholderText("Open left file...")
        self.left_text.setReadOnly(True)
        self.left_text.setStyleSheet(
            "QTextEdit { background-color: #ffffff; color: #111827; "
            "selection-background-color: #2563eb; selection-color: #ffffff; }"
        )
        self.left_text.setFocusPolicy(Qt.NoFocus)  # Disable focus highlighting
        self._apply_light_palette(self.left_text)
        self.left_panel.addWidget(self.left_text)
        # Keep scrolling in sync
        self.left_text.verticalScrollBar().valueChanged.connect(
            lambda value: self._sync_scroll(self.left_text, self.right_text, value)
        )
        # Right panel
        self.right_panel = QVBoxLayout()
        self.right_file_label = QLabel("No file selected")
        self.right_file_label.setStyleSheet(
            "QLabel { background-color: #f3f4f6; color: #374151; padding: 8px 12px; "
            "border-bottom: 1px solid #d1d5db; font-weight: 500; font-size: 12px; }"
        )
        self.right_panel.addWidget(self.right_file_label)
        self.right_text = QTextEdit()
        self.right_text.setPlaceholderText("Open right file...")
        self.right_text.setReadOnly(True)
        self.right_text.setStyleSheet(
            "QTextEdit { background-color: #ffffff; color: #111827; "
            "selection-background-color: #2563eb; selection-color: #ffffff; }"
        )
        self.right_text.setFocusPolicy(Qt.NoFocus)  # Disable focus highlighting
        self._apply_light_palette(self.right_text)
        self.right_panel.addWidget(self.right_text)
        # Keep scrolling in sync
        self.right_text.verticalScrollBar().valueChanged.connect(
            lambda value: self._sync_scroll(self.right_text, self.left_text, value)
        )
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
        self.left_file_label.setText(f"📄 {left_path}")
        # Apply syntax highlighting
        SyntaxHighlighter(self.left_text.document(), left_path)
        self._update_views()

    def load_right_file(self, right_path):
        with open(right_path, "r", encoding="utf-8", errors="ignore") as f:
            self.right_lines = f.readlines()
        self.right_file_label.setText(f"📄 {right_path}")
        # Apply syntax highlighting
        SyntaxHighlighter(self.right_text.document(), right_path)
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
                import difflib

                left_block = [line.rstrip("\n") for line in self.left_lines[i1:i2]]
                right_block = [line.rstrip("\n") for line in self.right_lines[j1:j2]]
                sub_matcher = difflib.SequenceMatcher(
                    None, left_block, right_block, autojunk=False
                )
                for stag, si1, si2, sj1, sj2 in sub_matcher.get_opcodes():
                    if stag == "equal":
                        for li, rj in zip(range(si1, si2), range(sj1, sj2)):
                            left_lines_out.append(left_block[li])
                            right_lines_out.append(right_block[rj])
                            left_formats.append(None)
                            right_formats.append(None)
                            region_map.append(None)
                    elif stag == "replace":
                        llen = si2 - si1
                        rlen = sj2 - sj1
                        maxlen = max(llen, rlen)
                        for k in range(maxlen):
                            left_line = left_block[si1 + k] if k < llen else ""
                            right_line = right_block[sj1 + k] if k < rlen else ""
                            left_lines_out.append(left_line)
                            right_lines_out.append(right_line)
                            region_map.append(None)
                            if left_line.rstrip() == right_line.rstrip():
                                left_formats.append(None)
                                right_formats.append(None)
                            elif left_line == "" and right_line != "":
                                left_formats.append(None)
                                right_formats.append(insert_format)
                            elif right_line == "" and left_line != "":
                                left_formats.append(delete_format)
                                right_formats.append(None)
                            else:
                                left_formats.append(change_format)
                                right_formats.append(change_format)
                    elif stag == "insert":
                        for k in range(sj1, sj2):
                            left_lines_out.append("")
                            left_formats.append(None)
                            right_lines_out.append(right_block[k])
                            right_formats.append(insert_format)
                            region_map.append(None)
                    elif stag == "delete":
                        for k in range(si1, si2):
                            left_lines_out.append(left_block[k])
                            left_formats.append(delete_format)
                            right_lines_out.append("")
                            right_formats.append(None)
                            region_map.append(None)
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

        # If aligned lines are identical, don't highlight them regardless of opcode
        for idx, (left_line, right_line) in enumerate(
            zip(left_lines_out, right_lines_out)
        ):
            if left_line.rstrip() == right_line.rstrip():
                left_formats[idx] = None
                right_formats[idx] = None

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

        self._change_line_indices = [
            idx
            for idx, (left_format, right_format) in enumerate(
                zip(left_formats, right_formats)
            )
            if left_format or right_format
        ]
        self._change_regions = self._build_regions(self._change_line_indices)
        self._current_region_index = -1
        self._current_region_lines = None

        # Highlight the first region if there are any changes
        if self._change_regions:
            start_line, end_line = self._change_regions[0]
            self._current_region_index = 0
            self._set_change_selection(start_line, end_line)
        else:
            self.left_text.setExtraSelections([])
            self.right_text.setExtraSelections([])

        # Reset scroll position to top after rendering
        self.left_text.verticalScrollBar().setValue(0)
        self.right_text.verticalScrollBar().setValue(0)

    def _build_regions(self, change_indices: list) -> list:
        """Group consecutive changed line indices into regions.

        Args:
            change_indices: List of line indices that have changes

        Returns:
            List of (start_line, end_line) tuples representing contiguous change regions
        """
        if not change_indices:
            return []

        regions = []
        region_start = change_indices[0]
        region_end = change_indices[0]

        for idx in change_indices[1:]:
            if idx == region_end + 1:
                # Consecutive index, extend current region
                region_end = idx
            else:
                # Gap found, save current region and start a new one
                regions.append((region_start, region_end))
                region_start = idx
                region_end = idx

        # Add the last region
        regions.append((region_start, region_end))
        return regions

    def _apply_light_palette(self, text_edit):
        palette = text_edit.palette()
        palette.setColor(QPalette.Base, QColor("#ffffff"))
        palette.setColor(QPalette.Text, QColor("#111827"))
        palette.setColor(QPalette.Highlight, QColor("#2563eb"))
        palette.setColor(QPalette.HighlightedText, QColor("#ffffff"))
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
        from PySide6.QtGui import QTextCharFormat

        cursor = text_edit.textCursor()
        cursor.movePosition(QTextCursor.MoveOperation.End)
        block_format = QTextBlockFormat()
        block_format.clearBackground()
        cursor.setBlockFormat(block_format)
        char_format = QTextCharFormat()
        if self.current_font:
            char_format.setFont(self.current_font)
        cursor.insertText(line, char_format)
        cursor.insertText("\n")
        text_edit.setTextCursor(cursor)

    def _sync_scroll(self, source_text, target_text, value):
        if self._is_syncing_scroll:
            return
        self._is_syncing_scroll = True
        try:
            source_bar = source_text.verticalScrollBar()
            target_bar = target_text.verticalScrollBar()
            source_max = max(source_bar.maximum(), 1)
            target_max = target_bar.maximum()
            ratio = value / source_max
            target_value = int(round(ratio * target_max))
            target_bar.setValue(target_value)
        finally:
            self._is_syncing_scroll = False

    def _scroll_to_line(self, line_index: int) -> None:
        """Scroll both text edits to show the given line with 3 lines of context above and below."""
        if line_index < 0:
            return

        context_offset = 3

        for text_edit in (self.left_text, self.right_text):
            doc = text_edit.document()
            total_blocks = doc.blockCount()

            # Determine scroll position with context on both sides
            scroll_target = max(0, line_index - context_offset)
            scroll_end = min(total_blocks - 1, line_index + context_offset)

            # Move cursor to scroll_end to ensure bottom context is visible
            end_block = doc.findBlockByNumber(scroll_end)
            if end_block.isValid():
                cursor = text_edit.textCursor()
                cursor.setPosition(end_block.position())
                text_edit.setTextCursor(cursor)
                text_edit.ensureCursorVisible()

            # Then move to scroll_target to show top context
            start_block = doc.findBlockByNumber(scroll_target)
            if start_block.isValid():
                cursor = text_edit.textCursor()
                cursor.setPosition(start_block.position())
                text_edit.setTextCursor(cursor)
                text_edit.ensureCursorVisible()

    def _set_change_selection(self, start_line: int, end_line: int) -> None:
        """Highlight a range of lines with the current region selection color."""
        from PySide6.QtGui import QTextCursor

        if self._current_region_lines == (start_line, end_line):
            return
        self._current_region_lines = (start_line, end_line)

        selection_format = QTextCharFormat()
        selection_format.setBackground(QColor("#bfdbfe"))
        selection_format.setProperty(QTextCharFormat.FullWidthSelection, True)

        for text_edit in (self.left_text, self.right_text):
            selections = []
            for line_idx in range(start_line, end_line + 1):
                block = text_edit.document().findBlockByNumber(line_idx)
                if not block.isValid():
                    continue
                cursor = QTextCursor(block)
                cursor.select(QTextCursor.LineUnderCursor)
                selection = QTextEdit.ExtraSelection()
                selection.cursor = cursor
                selection.format = selection_format
                selections.append(selection)
            text_edit.setExtraSelections(selections)

    def get_change_count(self) -> int:
        """Return the number of change regions."""
        return len(self._change_regions)

    def next_change(self) -> None:
        """Navigate to the next change region."""
        if not self._change_regions:
            return
        # Move to next region if not at the end
        if self._current_region_index < len(self._change_regions) - 1:
            self._current_region_index += 1
            start_line, end_line = self._change_regions[self._current_region_index]
            self._scroll_to_line(start_line)
            self._set_change_selection(start_line, end_line)

    def previous_change(self) -> None:
        """Navigate to the previous change region."""
        if not self._change_regions:
            return
        # Move to previous region if not at the beginning
        if self._current_region_index > 0:
            self._current_region_index -= 1
        elif self._current_region_index == -1:
            self._current_region_index = 0

        if self._current_region_index >= 0:
            start_line, end_line = self._change_regions[self._current_region_index]
            self._scroll_to_line(start_line)
            self._set_change_selection(start_line, end_line)

    def set_diff_engine(self, engine_name: str):
        """Set the diff engine to use (myers or smart)."""
        if engine_name in ("myers", "smart"):
            self.diff_engine = engine_name
            # Re-render with the new engine
            self._update_views()
