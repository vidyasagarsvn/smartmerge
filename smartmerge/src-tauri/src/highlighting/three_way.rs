use crate::models::{ThreeWayDiffResult, ThreeWayOpcode, ConflictType};
use super::styles::{ColorScheme, HighlightStyle, LineHighlight, Theme};
use serde::{Serialize, Deserialize};

/// Three-way merge highlight result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreeWayHighlightResult {
    /// Highlights for base version
    pub base_highlights: Vec<LineHighlight>,
    /// Highlights for left version (current/ours)
    pub left_highlights: Vec<LineHighlight>,
    /// Highlights for right version (incoming/theirs)
    pub right_highlights: Vec<LineHighlight>,
    /// Color scheme used
    pub color_scheme: ColorScheme,
    /// Theme used
    pub theme: Theme,
    /// Conflict markers and regions
    pub conflict_regions: Vec<ConflictRegion>,
}

/// Represents a conflict region in the merge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictRegion {
    /// Line number where conflict starts
    pub start_line: usize,
    /// Line number where conflict ends
    pub end_line: usize,
    /// Type of conflict
    pub conflict_type: ConflictType,
    /// Whether this conflict can be auto-merged
    pub auto_mergeable: bool,
    /// Description of the conflict
    pub description: String,
}

/// Highlighting mapper for three-way merges
pub struct ThreeWayHighlightMapper {
    color_scheme: ColorScheme,
    theme: Theme,
}

impl ThreeWayHighlightMapper {
    pub fn new(theme: Theme) -> Self {
        Self {
            color_scheme: ColorScheme::for_theme(theme),
            theme,
        }
    }

    /// Map a three-way diff result to highlights
    pub fn map_three_way(
        &self,
        merge_result: &ThreeWayDiffResult,
        _base_lines: &[String],
        _left_lines: &[String],
        _right_lines: &[String],
    ) -> ThreeWayHighlightResult {
        let mut result = ThreeWayHighlightResult {
            base_highlights: Vec::new(),
            left_highlights: Vec::new(),
            right_highlights: Vec::new(),
            color_scheme: self.color_scheme.clone(),
            theme: self.theme,
            conflict_regions: Vec::new(),
        };

        for opcode in &merge_result.opcodes {
            self.map_three_way_opcode(opcode, &mut result);
        }

        result
    }

    fn map_three_way_opcode(&self, opcode: &ThreeWayOpcode, result: &mut ThreeWayHighlightResult) {
        let is_conflict = opcode.conflict_type != ConflictType::None;

        // Determine colors based on conflict type
        let (bg, fg) = match opcode.conflict_type {
            ConflictType::None => {
                // No conflict - use normal change colors
                (&self.color_scheme.modifications.background, &self.color_scheme.modifications.text)
            }
            ConflictType::BothModified => {
                // Both sides modified - conflict
                (&self.color_scheme.conflicts.background, &self.color_scheme.conflicts.text)
            }
            ConflictType::DeleteModify | ConflictType::ModifyDelete => {
                // One side deleted, other modified - conflict
                (&self.color_scheme.conflicts.background, &self.color_scheme.conflicts.text)
            }
            ConflictType::BothAdded => {
                // Both sides added different content - conflict
                (&self.color_scheme.conflicts.background, &self.color_scheme.conflicts.text)
            }
        };

        // Highlight base lines
        for line_num in opcode.base_start..opcode.base_end {
            result.base_highlights.push(LineHighlight {
                line_number: line_num,
                style: HighlightStyle::new(bg.to_string(), fg.to_string()),
                block_type: crate::models::BlockType::Equal,
                inline_highlights: Vec::new(),
                is_conflict,
            });
        }

        // Highlight left lines
        for line_num in opcode.left_start..opcode.left_end {
            result.left_highlights.push(LineHighlight {
                line_number: line_num,
                style: HighlightStyle::new(
                    if is_conflict { bg.to_string() } else { self.color_scheme.modifications.background.clone() },
                    if is_conflict { fg.to_string() } else { self.color_scheme.modifications.text.clone() },
                ),
                block_type: crate::models::BlockType::Replace,
                inline_highlights: Vec::new(),
                is_conflict,
            });
        }

        // Highlight right lines
        for line_num in opcode.right_start..opcode.right_end {
            result.right_highlights.push(LineHighlight {
                line_number: line_num,
                style: HighlightStyle::new(
                    if is_conflict { bg.to_string() } else { self.color_scheme.modifications.background.clone() },
                    if is_conflict { fg.to_string() } else { self.color_scheme.modifications.text.clone() },
                ),
                block_type: crate::models::BlockType::Replace,
                inline_highlights: Vec::new(),
                is_conflict,
            });
        }

        // Track conflict regions
        if is_conflict {
            result.conflict_regions.push(ConflictRegion {
                start_line: opcode.left_start.min(opcode.right_start),
                end_line: opcode.left_end.max(opcode.right_end),
                conflict_type: opcode.conflict_type.clone(),
                auto_mergeable: opcode.auto_mergeable,
                description: self.describe_conflict(&opcode.conflict_type),
            });
        }
    }

    fn describe_conflict(&self, conflict_type: &ConflictType) -> String {
        match conflict_type {
            ConflictType::None => "No conflict".to_string(),
            ConflictType::BothModified => "Both sides modified the same section".to_string(),
            ConflictType::DeleteModify => "Left deleted while right modified".to_string(),
            ConflictType::ModifyDelete => "Left modified while right deleted".to_string(),
            ConflictType::BothAdded => "Both sides added different content".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ThreeWayDiffResult;

    #[test]
    fn test_three_way_mapper_creation() {
        let mapper = ThreeWayHighlightMapper::new(Theme::Light);
        assert_eq!(mapper.theme, Theme::Light);
    }

    #[test]
    fn test_map_empty_three_way() {
        let mapper = ThreeWayHighlightMapper::new(Theme::Dark);
        let merge_result = ThreeWayDiffResult {
            opcodes: vec![],
            conflicts: vec![],
            auto_merge_possible: true,
        };
        let result = mapper.map_three_way(&merge_result, &[], &[], &[]);
        assert!(result.base_highlights.is_empty());
        assert!(result.conflict_regions.is_empty());
    }

    #[test]
    fn test_conflict_description() {
        let mapper = ThreeWayHighlightMapper::new(Theme::Light);
        assert_eq!(
            mapper.describe_conflict(&ConflictType::BothModified),
            "Both sides modified the same section"
        );
    }
}
