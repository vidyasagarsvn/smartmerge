"""Line number area widget for text editors."""

from PySide6.QtWidgets import QWidget, QTextEdit
from PySide6.QtCore import Qt, QSize, QRect
from PySide6.QtGui import QPainter, QColor, QFont


class LineNumberArea(QWidget):
    """Display line numbers for a QTextEdit widget."""

    def __init__(self, text_edit: QTextEdit):
        super().__init__(text_edit)
        self.text_edit = text_edit
        self.text_edit.document().blockCountChanged.connect(self._update_width)
        self.text_edit.verticalScrollBar().valueChanged.connect(self.update)
        self._update_width()

    def sizeHint(self) -> QSize:
        """Return the recommended size for the line number area."""
        return QSize(self._width(), 0)

    def _width(self) -> int:
        """Calculate the width needed based on line count."""
        block_count = self.text_edit.document().blockCount()
        digits = len(str(max(1, block_count)))
        width = 35 + 10 * digits
        return width

    def _update_width(self):
        """Update the text edit's viewport margins when width changes."""
        self.text_edit.setViewportMargins(self._width(), 0, 0, 0)

    def paintEvent(self, event):
        """Paint line numbers."""
        painter = QPainter(self)

        # Background color
        painter.fillRect(event.rect(), QColor(240, 240, 240))

        # Text color
        painter.setPen(QColor(128, 128, 128))

        # Font
        font = self.text_edit.font()
        painter.setFont(font)

        # Get metrics
        block = self.text_edit.firstVisibleBlock()
        block_num = block.blockNumber()
        top = (
            self.text_edit.blockBoundingGeometry(block)
            .translated(self.text_edit.contentOffset())
            .top()
        )
        bottom = top + self.text_edit.blockBoundingRect(block).height()

        # Draw line numbers
        while block.isValid() and top <= event.rect().bottom():
            if block.isVisible() and bottom >= event.rect().top():
                text = str(block_num + 1)
                painter.drawText(
                    0,
                    int(top),
                    self.width() - 5,
                    int(self.text_edit.blockBoundingRect(block).height()),
                    Qt.AlignRight,
                    text,
                )

            block = block.next()
            block_num += 1
            top = bottom
            bottom = top + self.text_edit.blockBoundingRect(block).height()
