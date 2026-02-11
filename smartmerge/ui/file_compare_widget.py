
from PySide6.QtWidgets import QWidget, QHBoxLayout, QTextEdit, QSplitter, QVBoxLayout, QPushButton, QScrollArea, QFrame
from PySide6.QtCore import Qt
from PySide6.QtGui import QTextCharFormat, QColor
import difflib
import Levenshtein

class FileCompareWidget(QWidget):
    """Widget for displaying and comparing two files side by side with line-level diff highlighting."""
    def __init__(self, parent=None):
        super().__init__(parent)
        self.init_ui()
        self.left_lines = []
        self.right_lines = []

    def init_ui(self):
        layout = QHBoxLayout(self)
        # Left panel
        self.left_panel = QVBoxLayout()
        self.left_text = QTextEdit()
        self.left_text.setPlaceholderText("Open left file...")
        self.left_text.setReadOnly(True)
        self.left_panel.addWidget(self.left_text)
        # Right panel
        self.right_panel = QVBoxLayout()
        self.right_text = QTextEdit()
        self.right_text.setPlaceholderText("Open right file...")
        self.right_text.setReadOnly(True)
        self.right_panel.addWidget(self.right_text)
        # Merge controls panel
        self.controls_panel = QVBoxLayout()
        self.controls_panel.setAlignment(Qt.AlignTop)
        self.merge_buttons = []  # Store buttons for each diff line
        # Add panels to splitter
        splitter = QSplitter(Qt.Horizontal)
        left_widget = QWidget()
        left_widget.setLayout(self.left_panel)
        right_widget = QWidget()
        right_widget.setLayout(self.right_panel)
        controls_widget = QWidget()
        controls_widget.setLayout(self.controls_panel)
        splitter.addWidget(left_widget)
        splitter.addWidget(controls_widget)
        splitter.addWidget(right_widget)
        layout.addWidget(splitter)
        self.setLayout(layout)

    def load_left_file(self, left_path):
        with open(left_path, 'r', encoding='utf-8', errors='ignore') as f:
            self.left_lines = f.readlines()
        self._update_views()

    def load_right_file(self, right_path):
        with open(right_path, 'r', encoding='utf-8', errors='ignore') as f:
            self.right_lines = f.readlines()
        self._update_views()

    def _update_views(self):
        sm = difflib.SequenceMatcher(None, self.left_lines, self.right_lines)
        # Highlight formats
        change_format = QTextCharFormat()
        change_format.setBackground(QColor('#fff59d'))  # yellow
        insert_format = QTextCharFormat()
        insert_format.setBackground(QColor('#b2ffb2'))  # light green
        delete_format = QTextCharFormat()
        delete_format.setBackground(QColor('#ffb2b2'))  # light red
        # Clear previous merge controls
        for i in reversed(range(self.controls_panel.count())):
            widget = self.controls_panel.itemAt(i).widget()
            if widget:
                widget.setParent(None)
        left_lines_out = []
        right_lines_out = []
        left_formats = []
        right_formats = []
        line_map = []  # Track which lines are diff lines for merge controls
        region_map = []  # Store region diff info for each line (for region-based merge)
        # For each opcode, build output lines and region info
        for tag, i1, i2, j1, j2 in sm.get_opcodes():
            if tag == 'equal':
                for i, j in zip(range(i1, i2), range(j1, j2)):
                    left_lines_out.append(self.left_lines[i].rstrip('\n'))
                    right_lines_out.append(self.right_lines[j].rstrip('\n'))
                    left_formats.append(None)
                    right_formats.append(None)
                    line_map.append('equal')
                    region_map.append(None)
            elif tag == 'replace':
                llen = i2 - i1
                rlen = j2 - j1
                maxlen = max(llen, rlen)
                for k in range(maxlen):
                    left_line = self.left_lines[i1 + k].rstrip('\n') if k < llen else ""
                    right_line = self.right_lines[j1 + k].rstrip('\n') if k < rlen else ""
                    left_lines_out.append(left_line)
                    right_lines_out.append(right_line)
                    if left_line and right_line:
                        opcodes = Levenshtein.opcodes(left_line, right_line)
                        region_map.append(opcodes)
                        left_formats.append('region')
                        right_formats.append('region')
                    else:
                        region_map.append(None)
                        left_formats.append(change_format)
                        right_formats.append(change_format)
                    line_map.append('replace')
            elif tag == 'insert':
                for k in range(j1, j2):
                    left_lines_out.append("")
                    left_formats.append(insert_format)
                    right_lines_out.append(self.right_lines[k].rstrip('\n'))
                    right_formats.append(insert_format)
                    line_map.append('insert')
                    region_map.append(None)
            elif tag == 'delete':
                for k in range(i1, i2):
                    left_lines_out.append(self.left_lines[k].rstrip('\n'))
                    left_formats.append(delete_format)
                    right_lines_out.append("")
                    right_formats.append(delete_format)
                    line_map.append('delete')
                    region_map.append(None)
            elif tag == 'insert':
                for k in range(j1, j2):
                    left_lines_out.append("")
                    left_formats.append(insert_format)
                    right_lines_out.append(self.right_lines[k].rstrip('\n'))
                    right_formats.append(insert_format)
                    line_map.append('insert')
            elif tag == 'delete':
                for k in range(i1, i2):
                    left_lines_out.append(self.left_lines[k].rstrip('\n'))
                    left_formats.append(delete_format)
                    right_lines_out.append("")
                    right_formats.append(delete_format)
                    line_map.append('delete')

        # Output to text widgets
        self.left_text.clear()
        self.right_text.clear()
        # Output to text widgets and add merge controls
        self.left_text.clear()
        self.right_text.clear()
        for idx, (l, lf) in enumerate(zip(left_lines_out, left_formats)):
            if lf == 'region' and region_map[idx]:
                r = right_lines_out[idx]
                self._append_region_highlight(self.left_text, l, r, region_map[idx], is_left=True)
            elif lf:
                self._append_highlighted(self.left_text, l, lf)
            else:
                self.left_text.append(l)
        for idx, (r, rf) in enumerate(zip(right_lines_out, right_formats)):
            if rf == 'region' and region_map[idx]:
                l = left_lines_out[idx]
                self._append_region_highlight(self.right_text, r, l, region_map[idx], is_left=False)
            elif rf:
                self._append_highlighted(self.right_text, r, rf)
            else:
                self.right_text.append(r)

    def _append_region_highlight(self, text_edit, line, other_line, opcodes, is_left=True):
        from PySide6.QtGui import QTextCursor
        cursor = text_edit.textCursor()
        cursor.movePosition(QTextCursor.MoveOperation.End)
        for tag, i1, i2, j1, j2 in opcodes:
            frag = line[i1:i2] if is_left else other_line[j1:j2]
            frag_fmt = QTextCharFormat()
            if tag == 'replace':
                frag_fmt.setBackground(QColor('#ffecb3'))  # light orange for changed region
            elif tag == 'insert' and not is_left:
                frag_fmt.setBackground(QColor('#b2ffb2'))  # green for insert
            elif tag == 'delete' and is_left:
                frag_fmt.setBackground(QColor('#ffb2b2'))  # red for delete
            cursor.insertText(frag, frag_fmt)
        cursor.insertText('\n')
        text_edit.setTextCursor(cursor)
