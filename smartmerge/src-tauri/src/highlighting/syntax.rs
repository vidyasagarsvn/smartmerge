/// Syntax-aware highlighting support
/// 
/// This module provides syntax highlighting capabilities that can be
/// combined with diff highlighting for better code readability.

use serde::{Serialize, Deserialize};

/// Syntax highlighting token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxToken {
    /// Start position (character offset)
    pub start: usize,
    /// End position (character offset)
    pub end: usize,
    /// Token type (keyword, string, comment, etc.)
    pub token_type: String,
    /// Color for this token
    pub color: String,
}

/// Syntax highlighter
pub struct SyntaxHighlighter {
    // In a full implementation, this would use crates like:
    // - syntect for syntax highlighting
    // - tree-sitter for parsing
    // For now, we keep it as a placeholder
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        Self {}
    }

    /// Highlight a line of code
    pub fn highlight_line(&self, _line: &str, _language: &str) -> Vec<SyntaxToken> {
        // Placeholder implementation
        // A real implementation would:
        // 1. Parse the line using syntect or tree-sitter
        // 2. Generate tokens with appropriate colors
        // 3. Return the token list
        Vec::new()
    }

    /// Detect language from file extension
    pub fn detect_language(filename: &str) -> Option<String> {
        let ext = std::path::Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())?;
        
        match ext {
            "rs" => Some("rust".to_string()),
            "py" => Some("python".to_string()),
            "js" | "jsx" => Some("javascript".to_string()),
            "ts" | "tsx" => Some("typescript".to_string()),
            "cpp" | "cc" | "cxx" => Some("cpp".to_string()),
            "c" => Some("c".to_string()),
            "h" | "hpp" => Some("cpp".to_string()),
            "java" => Some("java".to_string()),
            "go" => Some("go".to_string()),
            "rb" => Some("ruby".to_string()),
            "php" => Some("php".to_string()),
            "swift" => Some("swift".to_string()),
            "kt" => Some("kotlin".to_string()),
            "cs" => Some("csharp".to_string()),
            "html" | "htm" => Some("html".to_string()),
            "css" => Some("css".to_string()),
            "json" => Some("json".to_string()),
            "xml" => Some("xml".to_string()),
            "yaml" | "yml" => Some("yaml".to_string()),
            "toml" => Some("toml".to_string()),
            "md" | "markdown" => Some("markdown".to_string()),
            "sh" | "bash" => Some("bash".to_string()),
            "sql" => Some("sql".to_string()),
            _ => None,
        }
    }
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_detection() {
        assert_eq!(SyntaxHighlighter::detect_language("main.rs"), Some("rust".to_string()));
        assert_eq!(SyntaxHighlighter::detect_language("script.py"), Some("python".to_string()));
        assert_eq!(SyntaxHighlighter::detect_language("App.tsx"), Some("typescript".to_string()));
        assert_eq!(SyntaxHighlighter::detect_language("unknown.xyz"), None);
    }

    #[test]
    fn test_highlighter_creation() {
        let highlighter = SyntaxHighlighter::new();
        let tokens = highlighter.highlight_line("fn main() {}", "rust");
        assert!(tokens.is_empty()); // Placeholder returns empty
    }
}
