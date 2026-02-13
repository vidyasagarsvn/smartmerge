use crate::models::ThreeWayDiffResult;
use super::state::{MergeAction, MergeState};
use super::operations::{MergeOperations, BlockVersion};
use serde::{Serialize, Deserialize};

/// Merge result with complete merged content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeResult {
    /// Final merged lines
    pub merged_lines: Vec<String>,
    /// Whether merge was successful (no unresolved conflicts)
    pub success: bool,
    /// Number of conflicts resolved
    pub conflicts_resolved: usize,
    /// Number of conflicts remaining
    pub conflicts_remaining: usize,
    /// Blocks that were merged
    pub merged_blocks: Vec<MergedBlockInfo>,
}

/// Information about a merged block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergedBlockInfo {
    pub block_index: usize,
    pub action: MergeAction,
    pub line_start: usize,
    pub line_end: usize,
}

/// Merge result builder
pub struct MergeBuilder<'a> {
    state: &'a MergeState,
    operations: MergeOperations<'a>,
}

impl<'a> MergeBuilder<'a> {
    pub fn new(
        state: &'a MergeState,
        diff_result: &'a ThreeWayDiffResult,
        base_lines: &'a [String],
        left_lines: &'a [String],
        right_lines: &'a [String],
    ) -> Self {
        Self {
            state,
            operations: MergeOperations::new(diff_result, base_lines, left_lines, right_lines),
        }
    }

    /// Build the final merge result
    pub fn build(&self) -> Result<MergeResult, String> {
        let mut merged_lines = Vec::new();
        let mut merged_blocks = Vec::new();
        let mut conflicts_resolved = 0;
        let mut conflicts_remaining = 0;

        for block_status in &self.state.blocks {
            let block_index = block_status.block_index;
            let line_start = merged_lines.len();

            match block_status.action {
                MergeAction::Pending | MergeAction::Skip => {
                    if block_status.has_conflict {
                        conflicts_remaining += 1;
                        // Add conflict markers
                        self.add_conflict_markers(
                            &mut merged_lines,
                            block_index,
                        )?;
                    }
                }
                MergeAction::AcceptLeft => {
                    let op = self.operations.accept_left(block_index)?;
                    merged_lines.extend(op.lines);
                    if block_status.has_conflict {
                        conflicts_resolved += 1;
                    }
                }
                MergeAction::AcceptRight => {
                    let op = self.operations.accept_right(block_index)?;
                    merged_lines.extend(op.lines);
                    if block_status.has_conflict {
                        conflicts_resolved += 1;
                    }
                }
                MergeAction::AcceptBase => {
                    let op = self.operations.accept_base(block_index)?;
                    merged_lines.extend(op.lines);
                    if block_status.has_conflict {
                        conflicts_resolved += 1;
                    }
                }
                MergeAction::Custom => {
                    if let Some(ref custom_lines) = block_status.custom_lines {
                        merged_lines.extend(custom_lines.clone());
                        if block_status.has_conflict {
                            conflicts_resolved += 1;
                        }
                    }
                }
                MergeAction::AutoMerged => {
                    // Use the auto-merged resolution if available
                    let lines = self.operations.get_block_lines(block_index, BlockVersion::Left)?;
                    merged_lines.extend(lines);
                }
            }

            let line_end = merged_lines.len();
            merged_blocks.push(MergedBlockInfo {
                block_index,
                action: block_status.action,
                line_start,
                line_end,
            });
        }

        Ok(MergeResult {
            merged_lines,
            success: conflicts_remaining == 0,
            conflicts_resolved,
            conflicts_remaining,
            merged_blocks,
        })
    }

    fn add_conflict_markers(
        &self,
        lines: &mut Vec<String>,
        block_index: usize,
    ) -> Result<(), String> {
        lines.push("<<<<<<< LEFT (ours)".to_string());
        
        let left_lines = self.operations.get_block_lines(block_index, BlockVersion::Left)?;
        lines.extend(left_lines);
        
        lines.push("=======".to_string());
        
        let right_lines = self.operations.get_block_lines(block_index, BlockVersion::Right)?;
        lines.extend(right_lines);
        
        lines.push(">>>>>>> RIGHT (theirs)".to_string());
        
        Ok(())
    }

    /// Build result with only resolved conflicts
    pub fn build_partial(&self) -> Result<MergeResult, String> {
        let mut merged_lines = Vec::new();
        let mut merged_blocks = Vec::new();
        let mut conflicts_resolved = 0;
        let mut conflicts_remaining = 0;

        for block_status in &self.state.blocks {
            let block_index = block_status.block_index;
            
            if block_status.action == MergeAction::Pending || block_status.action == MergeAction::Skip {
                if block_status.has_conflict {
                    conflicts_remaining += 1;
                }
                continue; // Skip unresolved blocks
            }

            let line_start = merged_lines.len();

            match block_status.action {
                MergeAction::AcceptLeft => {
                    let op = self.operations.accept_left(block_index)?;
                    merged_lines.extend(op.lines);
                    if block_status.has_conflict {
                        conflicts_resolved += 1;
                    }
                }
                MergeAction::AcceptRight => {
                    let op = self.operations.accept_right(block_index)?;
                    merged_lines.extend(op.lines);
                    if block_status.has_conflict {
                        conflicts_resolved += 1;
                    }
                }
                MergeAction::AcceptBase => {
                    let op = self.operations.accept_base(block_index)?;
                    merged_lines.extend(op.lines);
                    if block_status.has_conflict {
                        conflicts_resolved += 1;
                    }
                }
                MergeAction::Custom => {
                    if let Some(ref custom_lines) = block_status.custom_lines {
                        merged_lines.extend(custom_lines.clone());
                        if block_status.has_conflict {
                            conflicts_resolved += 1;
                        }
                    }
                }
                MergeAction::AutoMerged => {
                    let lines = self.operations.get_block_lines(block_index, BlockVersion::Left)?;
                    merged_lines.extend(lines);
                }
                _ => {}
            }

            let line_end = merged_lines.len();
            merged_blocks.push(MergedBlockInfo {
                block_index,
                action: block_status.action,
                line_start,
                line_end,
            });
        }

        Ok(MergeResult {
            merged_lines,
            success: conflicts_remaining == 0,
            conflicts_resolved,
            conflicts_remaining,
            merged_blocks,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ConflictType;

    #[test]
    fn test_build_simple_merge() {
        let opcodes = vec![ThreeWayOpcode {
            base_start: 0,
            base_end: 1,
            left_start: 0,
            left_end: 1,
            right_start: 0,
            right_end: 1,
            conflict_type: ConflictType::None,
            auto_mergeable: true,
            resolved_lines: None,
        }];

        let diff = ThreeWayDiffResult {
            opcodes,
            conflicts: vec![],
            auto_merge_possible: true,
        };

        let base = vec!["line 1".to_string()];
        let left = vec!["line 1 modified".to_string()];
        let right = vec!["line 1".to_string()];

        let mut state = MergeState::from_opcodes(&diff.opcodes);
        let builder = MergeBuilder::new(&state, &diff, &base, &left, &right);
        
        let result = builder.build().unwrap();
        assert!(result.success);
        assert_eq!(result.conflicts_remaining, 0);
    }
}
