from PySide6.QtCore import QRegularExpression
from PySide6.QtGui import (
    QSyntaxHighlighter,
    QTextDocument,
    QTextCharFormat,
    QColor,
    QFont,
)
from pathlib import Path


class SyntaxHighlighter(QSyntaxHighlighter):
    """Syntax highlighter for common programming languages."""

    def __init__(self, document: QTextDocument, file_path: str = None):
        super().__init__(document)
        self.file_path = file_path
        self.language = self._detect_language(file_path) if file_path else None
        self._setup_formats()

    def _detect_language(self, file_path: str) -> str:
        """Detect language from file extension."""
        if not file_path:
            return None
        ext = Path(file_path).suffix.lower()
        ext_map = {
            ".py": "python",
            ".js": "javascript",
            ".ts": "javascript",
            ".jsx": "javascript",
            ".tsx": "javascript",
            ".java": "java",
            ".c": "c",
            ".cpp": "cpp",
            ".cc": "cpp",
            ".h": "cpp",
            ".cs": "csharp",
            ".go": "go",
            ".rs": "rust",
            ".rb": "ruby",
            ".php": "php",
            ".swift": "swift",
            ".kt": "kotlin",
            ".scala": "scala",
            ".sh": "bash",
            ".json": "json",
            ".xml": "xml",
            ".html": "html",
            ".css": "css",
            ".sql": "sql",
            ".yaml": "yaml",
            ".yml": "yaml",
            ".md": "markdown",
        }
        return ext_map.get(ext)

    def _setup_formats(self):
        """Setup text formats for different token types."""
        self.formats = {}

        # Keywords
        keyword_format = QTextCharFormat()
        keyword_format.setForeground(QColor("#0066cc"))
        keyword_format.setFontWeight(QFont.Bold)
        self.formats["keyword"] = keyword_format

        # Strings
        string_format = QTextCharFormat()
        string_format.setForeground(QColor("#008000"))
        self.formats["string"] = string_format

        # Comments
        comment_format = QTextCharFormat()
        comment_format.setForeground(QColor("#808080"))
        comment_format.setFontItalic(True)
        self.formats["comment"] = comment_format

        # Numbers
        number_format = QTextCharFormat()
        number_format.setForeground(QColor("#dd7700"))
        self.formats["number"] = number_format

        # Functions/Methods
        function_format = QTextCharFormat()
        function_format.setForeground(QColor("#7f0055"))
        function_format.setFontWeight(QFont.Bold)
        self.formats["function"] = function_format

    def highlightBlock(self, text: str):
        """Highlight a block of text."""
        if not self.language:
            return

        if self.language == "python":
            self._highlight_python(text)
        elif self.language in ("javascript", "typescript"):
            self._highlight_javascript(text)
        elif self.language == "java":
            self._highlight_java(text)
        elif self.language in ("c", "cpp"):
            self._highlight_c(text)
        elif self.language == "json":
            self._highlight_json(text)
        elif self.language == "html":
            self._highlight_html(text)
        elif self.language == "sql":
            self._highlight_sql(text)

    def _highlight_python(self, text: str):
        """Highlight Python syntax."""
        keywords = [
            "and",
            "as",
            "assert",
            "break",
            "class",
            "continue",
            "def",
            "del",
            "elif",
            "else",
            "except",
            "False",
            "finally",
            "for",
            "from",
            "global",
            "if",
            "import",
            "in",
            "is",
            "lambda",
            "None",
            "not",
            "or",
            "pass",
            "raise",
            "return",
            "True",
            "try",
            "while",
            "with",
            "yield",
        ]

        # Highlight keywords
        for keyword in keywords:
            pattern = QRegularExpression(r"\b" + keyword + r"\b")
            iterator = pattern.globalMatch(text)
            while iterator.hasNext():
                match = iterator.next()
                self.setFormat(
                    match.capturedStart(),
                    match.capturedLength(),
                    self.formats["keyword"],
                )

        # Highlight strings (single and double quoted)
        for quote in ['"', "'"]:
            pattern = QRegularExpression(quote + r".*?" + quote)
            iterator = pattern.globalMatch(text)
            while iterator.hasNext():
                match = iterator.next()
                self.setFormat(
                    match.capturedStart(),
                    match.capturedLength(),
                    self.formats["string"],
                )

        # Highlight comments
        comment_start = text.find("#")
        if comment_start >= 0:
            self.setFormat(
                comment_start, len(text) - comment_start, self.formats["comment"]
            )

        # Highlight numbers
        pattern = QRegularExpression(r"\b\d+\.?\d*\b")
        iterator = pattern.globalMatch(text)
        while iterator.hasNext():
            match = iterator.next()
            self.setFormat(
                match.capturedStart(), match.capturedLength(), self.formats["number"]
            )

    def _highlight_javascript(self, text: str):
        """Highlight JavaScript/TypeScript syntax."""
        keywords = [
            "async",
            "await",
            "break",
            "case",
            "catch",
            "class",
            "const",
            "continue",
            "debugger",
            "default",
            "delete",
            "do",
            "else",
            "enum",
            "export",
            "extends",
            "false",
            "finally",
            "for",
            "from",
            "function",
            "if",
            "import",
            "in",
            "instanceof",
            "new",
            "null",
            "return",
            "super",
            "switch",
            "this",
            "throw",
            "true",
            "try",
            "typeof",
            "var",
            "void",
            "while",
            "with",
            "yield",
        ]

        for keyword in keywords:
            pattern = QRegularExpression(r"\b" + keyword + r"\b")
            iterator = pattern.globalMatch(text)
            while iterator.hasNext():
                match = iterator.next()
                self.setFormat(
                    match.capturedStart(),
                    match.capturedLength(),
                    self.formats["keyword"],
                )

        # Strings
        for quote in ['"', "'", "`"]:
            pattern = QRegularExpression(quote + r".*?" + quote)
            iterator = pattern.globalMatch(text)
            while iterator.hasNext():
                match = iterator.next()
                self.setFormat(
                    match.capturedStart(),
                    match.capturedLength(),
                    self.formats["string"],
                )

        # Comments
        if "//" in text:
            start = text.find("//")
            self.setFormat(start, len(text) - start, self.formats["comment"])

        # Numbers
        pattern = QRegularExpression(r"\b\d+\.?\d*\b")
        iterator = pattern.globalMatch(text)
        while iterator.hasNext():
            match = iterator.next()
            self.setFormat(
                match.capturedStart(), match.capturedLength(), self.formats["number"]
            )

    def _highlight_java(self, text: str):
        """Highlight Java syntax."""
        keywords = [
            "abstract",
            "assert",
            "boolean",
            "break",
            "byte",
            "case",
            "catch",
            "char",
            "class",
            "const",
            "continue",
            "default",
            "do",
            "double",
            "else",
            "enum",
            "extends",
            "false",
            "final",
            "finally",
            "float",
            "for",
            "goto",
            "if",
            "implements",
            "import",
            "instanceof",
            "int",
            "interface",
            "long",
            "native",
            "new",
            "null",
            "package",
            "private",
            "protected",
            "public",
            "return",
            "short",
            "static",
            "strictfp",
            "super",
            "switch",
            "synchronized",
            "this",
            "throw",
            "throws",
            "transient",
            "true",
            "try",
            "void",
            "volatile",
            "while",
        ]

        for keyword in keywords:
            pattern = QRegularExpression(r"\b" + keyword + r"\b")
            iterator = pattern.globalMatch(text)
            while iterator.hasNext():
                match = iterator.next()
                self.setFormat(
                    match.capturedStart(),
                    match.capturedLength(),
                    self.formats["keyword"],
                )

        # Strings
        for quote in ['"', "'"]:
            pattern = QRegularExpression(quote + r".*?" + quote)
            iterator = pattern.globalMatch(text)
            while iterator.hasNext():
                match = iterator.next()
                self.setFormat(
                    match.capturedStart(),
                    match.capturedLength(),
                    self.formats["string"],
                )

        # Comments
        if "//" in text:
            start = text.find("//")
            self.setFormat(start, len(text) - start, self.formats["comment"])

    def _highlight_c(self, text: str):
        """Highlight C/C++ syntax."""
        keywords = [
            "auto",
            "break",
            "case",
            "char",
            "const",
            "continue",
            "default",
            "do",
            "double",
            "else",
            "enum",
            "extern",
            "float",
            "for",
            "goto",
            "if",
            "inline",
            "int",
            "long",
            "register",
            "restrict",
            "return",
            "short",
            "signed",
            "sizeof",
            "static",
            "struct",
            "switch",
            "typedef",
            "union",
            "unsigned",
            "void",
            "volatile",
            "while",
            "class",
            "namespace",
            "template",
            "virtual",
        ]

        for keyword in keywords:
            pattern = QRegularExpression(r"\b" + keyword + r"\b")
            iterator = pattern.globalMatch(text)
            while iterator.hasNext():
                match = iterator.next()
                self.setFormat(
                    match.capturedStart(),
                    match.capturedLength(),
                    self.formats["keyword"],
                )

        # Strings
        for quote in ['"', "'"]:
            pattern = QRegularExpression(quote + r".*?" + quote)
            iterator = pattern.globalMatch(text)
            while iterator.hasNext():
                match = iterator.next()
                self.setFormat(
                    match.capturedStart(),
                    match.capturedLength(),
                    self.formats["string"],
                )

        # Comments
        if "//" in text:
            start = text.find("//")
            self.setFormat(start, len(text) - start, self.formats["comment"])

    def _highlight_json(self, text: str):
        """Highlight JSON syntax."""
        # Keys in quotes
        pattern = QRegularExpression(r'"[^"]*"(?=\s*:)')
        iterator = pattern.globalMatch(text)
        while iterator.hasNext():
            match = iterator.next()
            fmt = QTextCharFormat(self.formats["keyword"])
            self.setFormat(match.capturedStart(), match.capturedLength(), fmt)

        # String values
        pattern = QRegularExpression(r':\s*"[^"]*"')
        iterator = pattern.globalMatch(text)
        while iterator.hasNext():
            match = iterator.next()
            self.setFormat(
                match.capturedStart(), match.capturedLength(), self.formats["string"]
            )

        # Numbers
        pattern = QRegularExpression(r"\b\d+\.?\d*\b")
        iterator = pattern.globalMatch(text)
        while iterator.hasNext():
            match = iterator.next()
            self.setFormat(
                match.capturedStart(), match.capturedLength(), self.formats["number"]
            )

    def _highlight_html(self, text: str):
        """Highlight HTML syntax."""
        # Tags
        pattern = QRegularExpression(r"<[^>]*>")
        iterator = pattern.globalMatch(text)
        while iterator.hasNext():
            match = iterator.next()
            self.setFormat(
                match.capturedStart(), match.capturedLength(), self.formats["keyword"]
            )

        # Attributes
        pattern = QRegularExpression(r"\b\w+(?==)")
        iterator = pattern.globalMatch(text)
        while iterator.hasNext():
            match = iterator.next()
            self.setFormat(
                match.capturedStart(), match.capturedLength(), self.formats["function"]
            )

    def _highlight_sql(self, text: str):
        """Highlight SQL syntax."""
        keywords = [
            "SELECT",
            "FROM",
            "WHERE",
            "JOIN",
            "INNER",
            "LEFT",
            "RIGHT",
            "OUTER",
            "ON",
            "GROUP",
            "BY",
            "ORDER",
            "HAVING",
            "LIMIT",
            "OFFSET",
            "INSERT",
            "INTO",
            "VALUES",
            "UPDATE",
            "SET",
            "DELETE",
            "CREATE",
            "ALTER",
            "DROP",
            "TABLE",
            "DATABASE",
            "INDEX",
            "VIEW",
            "TRIGGER",
            "PROCEDURE",
            "FUNCTION",
            "AND",
            "OR",
            "NOT",
            "IN",
            "EXISTS",
            "BETWEEN",
            "LIKE",
            "NULL",
            "IS",
        ]

        for keyword in keywords:
            pattern = QRegularExpression(
                r"\b" + keyword + r"\b", QRegularExpression.CaseInsensitiveOption
            )
            iterator = pattern.globalMatch(text)
            while iterator.hasNext():
                match = iterator.next()
                self.setFormat(
                    match.capturedStart(),
                    match.capturedLength(),
                    self.formats["keyword"],
                )

        # Strings
        for quote in ['"', "'"]:
            pattern = QRegularExpression(quote + r".*?" + quote)
            iterator = pattern.globalMatch(text)
            while iterator.hasNext():
                match = iterator.next()
                self.setFormat(
                    match.capturedStart(),
                    match.capturedLength(),
                    self.formats["string"],
                )

        # Comments
        if "--" in text:
            start = text.find("--")
            self.setFormat(start, len(text) - start, self.formats["comment"])
