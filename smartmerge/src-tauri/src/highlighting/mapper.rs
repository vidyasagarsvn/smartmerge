use crate::models::{Opcode, BlockType, DiffResult};
use super::styles::{ColorScheme, HighlightStyle, LineHighlight, InlineHighlight, HighlightResult, Theme};

/// Highlighting mapper - converts diff opcodes to visual highlights
pub struct HighlightMapper {
    color_scheme: ColorScheme,
    pub theme: Theme,
    enable_inline_diff: bool,
}

impl HighlightMapper {
    pub fn new(theme: Theme) -> Self {
        Self {
            color_scheme: ColorScheme::for_theme(theme),
            theme,
            enable_inline_diff: true,
        }
    }

    pub fn with_inline_diff(mut self, enable: bool) -> Self {
        self.enable_inline_diff = enable;
        self
    }

    /// Map a DiffResult to a HighlightResult
    pub fn map_diff(&self, diff_result: &DiffResult, left_lines: &[String], right_lines: &[String]) -> HighlightResult {
        let mut result = HighlightResult::new(self.theme);
        result.color_scheme = self.color_scheme.clone();

        for opcode in &diff_result.opcodes {
            self.map_opcode(opcode, left_lines, right_lines, &mut result);
        }

        result
    }

    /// Map a single opcode to highlights
    fn map_opcode(&self, opcode: &Opcode, left_lines: &[String], right_lines: &[String], result: &mut HighlightResult) {
        let block_type = opcode.block_type.as_ref().unwrap_or(&BlockType::Equal);
        
        match block_type {
            BlockType::Equal => {
                // Equal blocks - low emphasis
                for line_num in opcode.i1..opcode.i2 {
                    result.left_highlights.push(self.create_line_highlight(
                        line_num,
                        &self.color_scheme.equal.background,
                        &self.color_scheme.equal.text,
                        BlockType::Equal,
                        false,
                    ));
                }
                for line_num in opcode.j1..opcode.j2 {
                    result.right_highlights.push(self.create_line_highlight(
                        line_num,
                        &self.color_scheme.equal.background,
                        &self.color_scheme.equal.text,
                        BlockType::Equal,
                        false,
                    ));
                }
            }
            BlockType::Insert => {
                // Inserted lines - show on right only
                let is_trivial = opcode.is_trivial.unwrap_or(false);
                let (bg, fg) = if is_trivial {
                    (&self.color_scheme.trivial_changes.background, &self.color_scheme.trivial_changes.text)
                } else {
                    (&self.color_scheme.additions.background, &self.color_scheme.additions.text)
                };

                for line_num in opcode.j1..opcode.j2 {
                    result.right_highlights.push(
                        self.create_line_highlight(line_num, bg, fg, BlockType::Insert, false)
                            .with_class("diff-insert")
                    );
                }
            }
            BlockType::Delete => {
                // Deleted lines - show on left only
                let is_trivial = opcode.is_trivial.unwrap_or(false);
                let (bg, fg) = if is_trivial {
                    (&self.color_scheme.trivial_changes.background, &self.color_scheme.trivial_changes.text)
                } else {
                    (&self.color_scheme.deletions.background, &self.color_scheme.deletions.text)
                };

                for line_num in opcode.i1..opcode.i2 {
                    result.left_highlights.push(
                        self.create_line_highlight(line_num, bg, fg, BlockType::Delete, false)
                            .with_class("diff-delete")
                    );
                }
            }
            BlockType::Replace => {
                // Changed lines - show on both sides with inline diff
                let is_trivial = opcode.is_trivial.unwrap_or(false);
                let (bg, fg) = if is_trivial {
                    (&self.color_scheme.trivial_changes.background, &self.color_scheme.trivial_changes.text)
                } else {
                    (&self.color_scheme.modifications.background, &self.color_scheme.modifications.text)
                };

                // Left side (deleted)
                for line_num in opcode.i1..opcode.i2 {
                    let mut highlight = self.create_line_highlight(line_num, bg, fg, BlockType::Replace, false)
                        .with_class("diff-replace");
                    
                    // Add inline diff for single-line changes
                    if self.enable_inline_diff 
                        && (opcode.i2 - opcode.i1) == 1 
                        && (opcode.j2 - opcode.j1) == 1 {
                        if let Some(left_line) = left_lines.get(line_num) {
                            if let Some(right_line) = right_lines.get(opcode.j1) {
                                highlight.inline_highlights = self.compute_inline_diff(left_line, right_line);
                            }
                        }
                    }
                    
                    result.left_highlights.push(highlight);
                }

                // Right side (added)
                for line_num in opcode.j1..opcode.j2 {
                    let mut highlight = self.create_line_highlight(line_num, bg, fg, BlockType::Replace, false)
                        .with_class("diff-replace");
                    
                    // Add inline diff for single-line changes
                    if self.enable_inline_diff 
                        && (opcode.i2 - opcode.i1) == 1 
                        && (opcode.j2 - opcode.j1) == 1 {
                        if let Some(left_line) = left_lines.get(opcode.i1) {
                            if let Some(right_line) = right_lines.get(line_num) {
                                highlight.inline_highlights = self.compute_inline_diff(left_line, right_line);
                            }
                        }
                    }
                    
                    result.right_highlights.push(highlight);
                }
            }
            BlockType::Moved => {
                // Moved blocks
                let bg = &self.color_scheme.moved_blocks.background;
                let fg = &self.color_scheme.moved_blocks.text;

                for line_num in opcode.i1..opcode.i2 {
                    result.left_highlights.push(
                        self.create_line_highlight(line_num, bg, fg, BlockType::Moved, false)
                            .with_class("diff-moved")
                    );
                }
                for line_num in opcode.j1..opcode.j2 {
                    result.right_highlights.push(
                        self.create_line_highlight(line_num, bg, fg, BlockType::Moved, false)
                            .with_class("diff-moved")
                    );
                }
            }
        }
    }

    /// Create a line highlight
    fn create_line_highlight(&self, line_number: usize, bg: &str, fg: &str, block_type: BlockType, is_conflict: bool) -> LineHighlight {
        LineHighlight {
            line_number,
            style: HighlightStyle::new(bg.to_string(), fg.to_string()),
            block_type,
            inline_highlights: Vec::new(),
            is_conflict,
        }
    }

    /// Compute inline/word-level diff for two lines
    fn compute_inline_diff(&self, left: &str, right: &str) -> Vec<InlineHighlight> {
        let mut highlights = Vec::new();
        
        // Simple word-level diff using character-by-character comparison
        let left_chars: Vec<char> = left.chars().collect();
        let right_chars: Vec<char> = right.chars().collect();
        
        let mut in_diff = false;
        let mut diff_start = 0;
        
        for (i, (lc, rc)) in left_chars.iter().zip(right_chars.iter()).enumerate() {
            if lc != rc {
                if !in_diff {
                    diff_start = i;
                    in_diff = true;
                }
            } else if in_diff {
                highlights.push(InlineHighlight {
                    start: diff_start,
                    end: i,
                    background: self.color_scheme.inline_highlight_bg.clone(),
                });
                in_diff = false;
            }
        }
        
        // Handle trailing differences
        if in_diff {
            highlights.push(InlineHighlight {
                start: diff_start,
                end: left_chars.len().max(right_chars.len()),
                background: self.color_scheme.inline_highlight_bg.clone(),
            });
        }
        
        // Handle length differences
        if left_chars.len() != right_chars.len() {
            let min_len = left_chars.len().min(right_chars.len());
            let max_len = left_chars.len().max(right_chars.len());
            if min_len < max_len {
                highlights.push(InlineHighlight {
                    start: min_len,
                    end: max_len,
                    background: self.color_scheme.inline_highlight_bg.clone(),
                });
            }
        }
        
        highlights
    }
}

/// Helper extension trait for LineHighlight
impl LineHighlight {
    pub fn with_class(mut self, class: &str) -> Self {
        self.style.css_classes.push(class.to_string());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mapper_creation() {
        let mapper = HighlightMapper::new(Theme::Light);
        assert_eq!(mapper.theme, Theme::Light);
        assert!(mapper.enable_inline_diff);
    }

    #[test]
    fn test_inline_diff_computation() {
        let mapper = HighlightMapper::new(Theme::Light);
        let highlights = mapper.compute_inline_diff("hello world", "hello Rust!");
        assert!(!highlights.is_empty());
    }

    #[test]
    fn test_map_empty_diff() {
        let mapper = HighlightMapper::new(Theme::Dark);
        let diff_result = DiffResult {
            opcodes: vec![],
            total_changes: 0,
            algorithm_used: "myers".to_string(),
            moved_blocks: vec![],
            options_used: None,
        };
        let result = mapper.map_diff(&diff_result, &[], &[]);
        assert!(result.left_highlights.is_empty());
        assert!(result.right_highlights.is_empty());
    }
}
