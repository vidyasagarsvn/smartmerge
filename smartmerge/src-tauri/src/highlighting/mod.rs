/// Highlighting engine for SmartMerge
/// 
/// Provides visual highlighting for diff results with support for:
/// - Light and dark themes
/// - Block-level highlighting (added, deleted, changed, moved)
/// - Inline/word-level diff highlighting
/// - Syntax-aware highlighting
/// - Three-way merge conflict highlighting

mod styles;
mod mapper;
mod syntax;
mod three_way;

pub use styles::{
    ColorScheme,
    HighlightResult,
    Theme,
};

pub use mapper::HighlightMapper;
pub use syntax::SyntaxHighlighter;
pub use three_way::{ThreeWayHighlightMapper, ThreeWayHighlightResult};

/// Main highlighting engine
pub struct HighlightEngine {
    mapper: HighlightMapper,
    syntax_highlighter: Option<SyntaxHighlighter>,
}

impl HighlightEngine {
    /// Create new highlighting engine with theme
    pub fn new(theme: Theme) -> Self {
        Self {
            mapper: HighlightMapper::new(theme),
            syntax_highlighter: None,
        }
    }

    /// Create highlighting engine with syntax support
    pub fn with_syntax(theme: Theme) -> Self {
        Self {
            mapper: HighlightMapper::new(theme),
            syntax_highlighter: Some(SyntaxHighlighter::new()),
        }
    }

    /// Enable or disable inline diff highlighting
    pub fn set_inline_diff(&mut self, enable: bool) {
        self.mapper = HighlightMapper::new(self.mapper.theme).with_inline_diff(enable);
    }

    /// Highlight a diff result
    pub fn highlight(&self, diff_result: &crate::models::DiffResult, left_lines: &[String], right_lines: &[String]) -> HighlightResult {
        let result = self.mapper.map_diff(diff_result, left_lines, right_lines);
        
        // Apply syntax highlighting if enabled
        if let Some(ref _syntax) = self.syntax_highlighter {
            // Note: Syntax highlighting would be integrated here
            // For now, we just return the diff-based highlighting
        }
        
        result
    }

    /// Highlight a three-way merge
    pub fn highlight_three_way(
        &self,
        merge_result: &crate::models::ThreeWayDiffResult,
        base_lines: &[String],
        left_lines: &[String],
        right_lines: &[String],
    ) -> ThreeWayHighlightResult {
        let mapper = ThreeWayHighlightMapper::new(self.mapper.theme);
        mapper.map_three_way(merge_result, base_lines, left_lines, right_lines)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::DiffResult;

    #[test]
    fn test_engine_creation() {
        let engine = HighlightEngine::new(Theme::Light);
        assert!(engine.syntax_highlighter.is_none());
    }

    #[test]
    fn test_engine_with_syntax() {
        let engine = HighlightEngine::with_syntax(Theme::Dark);
        assert!(engine.syntax_highlighter.is_some());
    }

    #[test]
    fn test_highlight_empty() {
        let engine = HighlightEngine::new(Theme::Light);
        let diff = DiffResult {
            opcodes: vec![],
            total_changes: 0,
            algorithm_used: "myers".to_string(),
            moved_blocks: vec![],
            options_used: None,
        };
        let result = engine.highlight(&diff, &[], &[]);
        assert!(result.left_highlights.is_empty());
    }
}
