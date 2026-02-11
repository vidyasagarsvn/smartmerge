from PySide6.QtWidgets import (
    QWidget,
    QHBoxLayout,
    QTextEdit,
    QSplitter,
    QVBoxLayout,
    QLabel,
    QPushButton,
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
        self.left_file_path = None
        self.right_file_path = None
        self.diff_engine = "smart"  # Default to Smart Block Diff engine
        self._is_syncing_scroll = False
        self._change_regions = []  # List of (start_line, end_line) tuples for each change region
        self._region_file_ranges = []  # List of ((left_start, left_end), (right_start, right_end)) for each region
        self._current_region_index = -1
        self._current_region_lines = None
        self._left_line_map = []  # Maps rendered line index to original left_lines index
        self._right_line_map = []  # Maps rendered line index to original right_lines index

        # Track unsaved state
        self.is_left_modified = False
        self.is_right_modified = False

        # Undo/Redo stacks - each entry is a tuple of (left_lines_copy, right_lines_copy)
        self.undo_stack = []
        self.redo_stack = []
        self.max_undo_history = 50

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
        controls_widget.setFixedWidth(60)
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
        self.left_file_path = left_path
        self.left_file_label.setText(f"📄 {left_path}")
        # Apply syntax highlighting
        SyntaxHighlighter(self.left_text.document(), left_path)
        self.is_left_modified = False
        self.undo_stack.clear()
        self.redo_stack.clear()
        self._current_region_index = -1
        self._update_views()
        self._select_first_region()

    def load_right_file(self, right_path):
        with open(right_path, "r", encoding="utf-8", errors="ignore") as f:
            self.right_lines = f.readlines()
        self.right_file_path = right_path
        self.right_file_label.setText(f"📄 {right_path}")
        # Apply syntax highlighting
        SyntaxHighlighter(self.right_text.document(), right_path)
        self.is_right_modified = False
        self.undo_stack.clear()
        self.redo_stack.clear()
        self._current_region_index = -1
        self._update_views()
        self._select_first_region()

    def _update_file_labels(self) -> None:
        """Update file labels to show unsaved status."""
        left_indicator = " *" if self.is_left_modified else ""
        right_indicator = " *" if self.is_right_modified else ""
        if self.left_file_path:
            self.left_file_label.setText(f"📄 {self.left_file_path}{left_indicator}")
        if self.right_file_path:
            self.right_file_label.setText(f"📄 {self.right_file_path}{right_indicator}")

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
        # Line highlight formats with subtle but clear colors
        change_format = QTextCharFormat()
        change_format.setBackground(QColor("#fef3c7"))  # changed (light amber)
        insert_format = QTextCharFormat()
        insert_format.setBackground(QColor("#dcfce7"))  # added (light green)
        delete_format = QTextCharFormat()
        delete_format.setBackground(QColor("#fee2e2"))  # deleted (light red)
        left_lines_out = []
        right_lines_out = []
        left_formats = []
        right_formats = []
        self._left_line_map = []  # Track original left_lines index for each rendered line
        self._right_line_map = []  # Track original right_lines index for each rendered line
        region_map = []  # Store region diff info for each line (unused for full-line)
        # For each opcode, build output lines and region info
        for tag, i1, i2, j1, j2 in opcodes:
            if tag == "equal":
                for i, j in zip(range(i1, i2), range(j1, j2)):
                    left_lines_out.append(self.left_lines[i].rstrip("\n"))
                    right_lines_out.append(self.right_lines[j].rstrip("\n"))
                    left_formats.append(None)
                    right_formats.append(None)
                    self._left_line_map.append(i)
                    self._right_line_map.append(j)
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
                            self._left_line_map.append(i1 + li)
                            self._right_line_map.append(j1 + rj)
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
                            self._left_line_map.append(i1 + si1 + k if k < llen else -1)
                            self._right_line_map.append(
                                j1 + sj1 + k if k < rlen else -1
                            )
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
                            self._left_line_map.append(-1)
                            self._right_line_map.append(j1 + k)
                            region_map.append(None)
                    elif stag == "delete":
                        for k in range(si1, si2):
                            left_lines_out.append(left_block[k])
                            left_formats.append(delete_format)
                            right_lines_out.append("")
                            right_formats.append(None)
                            self._left_line_map.append(i1 + k)
                            self._right_line_map.append(-1)
                            region_map.append(None)
            elif tag == "insert":
                for k in range(j1, j2):
                    left_lines_out.append("")
                    left_formats.append(None)
                    right_lines_out.append(self.right_lines[k].rstrip("\n"))
                    right_formats.append(insert_format)
                    self._left_line_map.append(-1)
                    self._right_line_map.append(k)
                    region_map.append(None)
            elif tag == "delete":
                for k in range(i1, i2):
                    left_lines_out.append(self.left_lines[k].rstrip("\n"))
                    left_formats.append(delete_format)
                    right_lines_out.append("")
                    right_formats.append(None)
                    self._left_line_map.append(k)
                    self._right_line_map.append(-1)
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
            # Add line number prefix
            line_num_text = f"{idx + 1:4d} | "
            if left_format:
                self._append_highlighted(
                    self.left_text, line_num_text + left_line_text, left_format
                )
            else:
                self._append_plain(self.left_text, line_num_text + left_line_text)
        for idx, (right_line_text, right_format) in enumerate(
            zip(right_lines_out, right_formats)
        ):
            # Add line number prefix
            line_num_text = f"{idx + 1:4d} | "
            if right_format:
                self._append_highlighted(
                    self.right_text, line_num_text + right_line_text, right_format
                )
            else:
                self._append_plain(self.right_text, line_num_text + right_line_text)

        self._change_line_indices = [
            idx
            for idx, (left_format, right_format) in enumerate(
                zip(left_formats, right_formats)
            )
            if left_format or right_format
        ]
        self._change_regions = self._build_regions(self._change_line_indices)
        self._region_file_ranges = self._build_region_file_ranges(self._change_regions)
        self._current_region_lines = None

        # Preserve current region index - clamp to valid range
        if self._change_regions:
            # Keep index in valid range [0, len-1]
            if self._current_region_index < 0:
                self._current_region_index = 0
            elif self._current_region_index >= len(self._change_regions):
                self._current_region_index = len(self._change_regions) - 1
        else:
            # No regions exist - clear selections and reset index
            self.left_text.setExtraSelections([])
            self.right_text.setExtraSelections([])
            self._current_region_index = -1

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

    def _build_region_file_ranges(self, regions: list) -> list:
        """Build the original file line ranges for each region.

        Args:
            regions: List of (start_line, end_line) tuples for rendered lines

        Returns:
            List of ((left_start, left_end), (right_start, right_end)) for original files
        """
        file_ranges = []
        for start_line, end_line in regions:
            left_indices = set()
            right_indices = set()

            for i in range(start_line, end_line + 1):
                if i < len(self._left_line_map) and self._left_line_map[i] >= 0:
                    left_indices.add(self._left_line_map[i])
                if i < len(self._right_line_map) and self._right_line_map[i] >= 0:
                    right_indices.add(self._right_line_map[i])

            # Get contiguous ranges
            left_range = (
                (min(left_indices), max(left_indices)) if left_indices else (-1, -1)
            )
            right_range = (
                (min(right_indices), max(right_indices)) if right_indices else (-1, -1)
            )

            file_ranges.append((left_range, right_range))

        return file_ranges

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

    def _select_first_region(self) -> None:
        """Select the first change region if it exists."""
        if self._change_regions and self._current_region_index < 0:
            self._current_region_index = 0
            start_line, end_line = self._change_regions[0]
            self._scroll_to_line(start_line)
            self._set_change_selection(start_line, end_line)

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

    def copy_to_right(self) -> None:
        """Copy current region from left to right."""
        if not self._change_regions or self._current_region_index < 0:
            return
        if self._current_region_index >= len(self._region_file_ranges):
            return

        # Save current state to undo stack before making changes
        self._push_undo_state()
        self.redo_stack.clear()

        (left_start, left_end), (right_start, right_end) = self._region_file_ranges[
            self._current_region_index
        ]

        # Get the rendered line position to help determine insertion point
        rendered_start, rendered_end = self._change_regions[self._current_region_index]

        # Copy, insert, or delete
        if left_start >= 0 and left_end >= 0:
            # Left has lines
            if right_start >= 0 and right_end >= 0:
                # Right also has lines - replace them with left
                left_lines = self.left_lines[left_start : left_end + 1]
                self.right_lines[right_start : right_end + 1] = left_lines
            elif right_start < 0:
                # Right has no lines (left insertion) - insert left lines to right
                left_lines = self.left_lines[left_start : left_end + 1]
                insert_pos = len(self.right_lines)  # Default: end of file

                # Find the next valid right position from the right_line_map
                for i in range(rendered_end + 1, len(self._right_line_map)):
                    if self._right_line_map[i] >= 0:
                        insert_pos = self._right_line_map[i]
                        break

                # Insert at the calculated position
                self.right_lines[insert_pos:insert_pos] = left_lines
        elif left_start < 0 and right_start >= 0 and right_end >= 0:
            # Right has lines but left doesn't (right insertion) - delete from right
            del self.right_lines[right_start : right_end + 1]

        self.is_right_modified = True
        self._update_file_labels()
        self._update_views()

        # After update, the copied region is gone, so current index points to next region
        if (
            self._change_regions
            and self._current_region_index >= 0
            and self._current_region_index < len(self._change_regions)
        ):
            start_line, end_line = self._change_regions[self._current_region_index]
            self._scroll_to_line(start_line)
            self._set_change_selection(start_line, end_line)

    def copy_to_left(self) -> None:
        """Copy current region from right to left."""
        if not self._change_regions or self._current_region_index < 0:
            return
        if self._current_region_index >= len(self._region_file_ranges):
            return

        # Save current state to undo stack before making changes
        self._push_undo_state()
        self.redo_stack.clear()

        (left_start, left_end), (right_start, right_end) = self._region_file_ranges[
            self._current_region_index
        ]

        # Get the rendered line position to help determine insertion point
        rendered_start, rendered_end = self._change_regions[self._current_region_index]

        # Copy, insert, or delete
        if right_start >= 0 and right_end >= 0:
            # Right has lines
            if left_start >= 0 and left_end >= 0:
                # Left also has lines - replace them with right
                right_lines = self.right_lines[right_start : right_end + 1]
                self.left_lines[left_start : left_end + 1] = right_lines
            elif left_start < 0:
                # Left has no lines (right insertion) - insert right lines to left
                right_lines = self.right_lines[right_start : right_end + 1]
                insert_pos = len(self.left_lines)  # Default: end of file

                # Find the next valid left position from the left_line_map
                for i in range(rendered_end + 1, len(self._left_line_map)):
                    if self._left_line_map[i] >= 0:
                        insert_pos = self._left_line_map[i]
                        break

                # Insert at the calculated position
                self.left_lines[insert_pos:insert_pos] = right_lines
        elif right_start < 0 and left_start >= 0 and left_end >= 0:
            # Left has lines but right doesn't (left insertion) - delete from left
            del self.left_lines[left_start : left_end + 1]

        self.is_left_modified = True
        self._update_file_labels()
        self._update_views()

        # After update, the copied region is gone, so current index points to next region
        if (
            self._change_regions
            and self._current_region_index >= 0
            and self._current_region_index < len(self._change_regions)
        ):
            start_line, end_line = self._change_regions[self._current_region_index]
            self._scroll_to_line(start_line)
            self._set_change_selection(start_line, end_line)

    def _push_undo_state(self) -> None:
        """Save current state to undo stack."""
        # Store copies of the current lines and current region index
        state = (
            self.left_lines.copy(),
            self.right_lines.copy(),
            self._current_region_index,
        )
        self.undo_stack.append(state)

        # Limit undo history to max_undo_history
        if len(self.undo_stack) > self.max_undo_history:
            self.undo_stack.pop(0)

    def undo(self) -> None:
        """Undo the last copy operation."""
        if not self.undo_stack:
            return

        # Save current state to redo stack
        current_state = (
            self.left_lines.copy(),
            self.right_lines.copy(),
            self._current_region_index,
        )
        self.redo_stack.append(current_state)

        # Restore previous state
        left_lines, right_lines, region_index = self.undo_stack.pop()
        self.left_lines = left_lines
        self.right_lines = right_lines

        # Reload files to check if there are unsaved changes
        if self.left_file_path:
            try:
                with open(
                    self.left_file_path, "r", encoding="utf-8", errors="ignore"
                ) as f:
                    saved_left_lines = f.readlines()
                self.is_left_modified = self.left_lines != saved_left_lines
            except:
                self.is_left_modified = True

        if self.right_file_path:
            try:
                with open(
                    self.right_file_path, "r", encoding="utf-8", errors="ignore"
                ) as f:
                    saved_right_lines = f.readlines()
                self.is_right_modified = self.right_lines != saved_right_lines
            except:
                self.is_right_modified = True

        self._update_file_labels()
        self._update_views()

        # Restore the region selection that was affected
        self._current_region_index = region_index
        if 0 <= self._current_region_index < len(self._change_regions):
            start_line, end_line = self._change_regions[self._current_region_index]
            self._scroll_to_line(start_line)
            self._set_change_selection(start_line, end_line)

    def redo(self) -> None:
        """Redo the last undone operation."""
        if not self.redo_stack:
            return

        # Save current state to undo stack
        current_state = (
            self.left_lines.copy(),
            self.right_lines.copy(),
            self._current_region_index,
        )
        self.undo_stack.append(current_state)

        # Restore redo state
        left_lines, right_lines, region_index = self.redo_stack.pop()
        self.left_lines = left_lines
        self.right_lines = right_lines

        # Reload files to check if there are unsaved changes
        if self.left_file_path:
            try:
                with open(
                    self.left_file_path, "r", encoding="utf-8", errors="ignore"
                ) as f:
                    saved_left_lines = f.readlines()
                self.is_left_modified = self.left_lines != saved_left_lines
            except:
                self.is_left_modified = True

        if self.right_file_path:
            try:
                with open(
                    self.right_file_path, "r", encoding="utf-8", errors="ignore"
                ) as f:
                    saved_right_lines = f.readlines()
                self.is_right_modified = self.right_lines != saved_right_lines
            except:
                self.is_right_modified = True

        self._update_file_labels()
        self._update_views()

        # Restore the region selection that was affected
        self._current_region_index = region_index
        if 0 <= self._current_region_index < len(self._change_regions):
            start_line, end_line = self._change_regions[self._current_region_index]
            self._scroll_to_line(start_line)
            self._set_change_selection(start_line, end_line)

    def save_left_file(self) -> bool:
        """Save the left file. Returns True if successful."""
        if not self.left_file_path:
            return False

        try:
            with open(self.left_file_path, "w", encoding="utf-8") as f:
                f.writelines(self.left_lines)
            self.is_left_modified = False
            self._update_file_labels()
            return True
        except Exception as e:
            print(f"Error saving left file: {e}")
            return False

    def save_right_file(self) -> bool:
        """Save the right file. Returns True if successful."""
        if not self.right_file_path:
            return False

        try:
            with open(self.right_file_path, "w", encoding="utf-8") as f:
                f.writelines(self.right_lines)
            self.is_right_modified = False
            self._update_file_labels()
            return True
        except Exception as e:
            print(f"Error saving right file: {e}")
            return False

    def can_undo(self) -> bool:
        """Check if undo is available."""
        return len(self.undo_stack) > 0

    def can_redo(self) -> bool:
        """Check if redo is available."""
        return len(self.redo_stack) > 0

    def get_status_info(self) -> str:
        """Return status information string for display."""
        status_parts = []

        # Region information
        if self._change_regions:
            region_num = self._current_region_index + 1
            total_regions = len(self._change_regions)
            status_parts.append(f"Region {region_num}/{total_regions}")
        else:
            status_parts.append("No differences")
            return " | ".join(status_parts)

        # File sizes
        import os

        if self.left_file_path and os.path.exists(self.left_file_path):
            left_size = os.path.getsize(self.left_file_path)
            status_parts.append(f"Left: {self._format_size(left_size)}")

        if self.right_file_path and os.path.exists(self.right_file_path):
            right_size = os.path.getsize(self.right_file_path)
            status_parts.append(f"Right: {self._format_size(right_size)}")

        # Line counts
        status_parts.append(f"Lines: {len(self.left_lines)} vs {len(self.right_lines)}")

        return " | ".join(status_parts)

    def _format_size(self, size: int) -> str:
        """Format file size in human-readable format."""
        for unit in ["B", "KB", "MB", "GB"]:
            if size < 1024:
                return f"{size:.1f}{unit}"
            size /= 1024
        return f"{size:.1f}TB"

    def set_diff_engine(self, engine_name: str):
        """Set the diff engine to use (myers or smart)."""
        if engine_name in ("myers", "smart"):
            self.diff_engine = engine_name
            # Re-render with the new engine
            self._update_views()
