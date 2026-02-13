use crate::models::{ThreeWayOpcode, ThreeWayDiffResult};
use super::state::MergeAction;

/// Merge operation result
#[derive(Debug, Clone)]
pub struct MergeOperation {
    /// Block index
    pub block_index: usize,
    /// Action to apply
    pub action: MergeAction,
    /// Lines to include in result
    pub lines: Vec<String>,
}

/// Merge operations handler
pub struct MergeOperations<'a> {
    opcodes: &'a [ThreeWayOpcode],
    base_lines: &'a [String],
    left_lines: &'a [String],
    right_lines: &'a [String],
}

impl<'a> MergeOperations<'a> {
    pub fn new(
        diff_result: &'a ThreeWayDiffResult,
        base_lines: &'a [String],
        left_lines: &'a [String],
        right_lines: &'a [String],
    ) -> Self {
        Self {
            opcodes: &diff_result.opcodes,
            base_lines,
            left_lines,
            right_lines,
        }
    }

    /// Accept left version of a block
    pub fn accept_left(&self, block_index: usize) -> Result<MergeOperation, String> {
        let opcode = self.get_opcode(block_index)?;
        let lines = self.left_lines[opcode.left_start..opcode.left_end].to_vec();

        Ok(MergeOperation {
            block_index,
            action: MergeAction::AcceptLeft,
            lines,
        })
    }

    /// Accept right version of a block
    pub fn accept_right(&self, block_index: usize) -> Result<MergeOperation, String> {
        let opcode = self.get_opcode(block_index)?;
        let lines = self.right_lines[opcode.right_start..opcode.right_end].to_vec();

        Ok(MergeOperation {
            block_index,
            action: MergeAction::AcceptRight,
            lines,
        })
    }

    /// Accept base version of a block
    pub fn accept_base(&self, block_index: usize) -> Result<MergeOperation, String> {
        let opcode = self.get_opcode(block_index)?;
        let lines = self.base_lines[opcode.base_start..opcode.base_end].to_vec();

        Ok(MergeOperation {
            block_index,
            action: MergeAction::AcceptBase,
            lines,
        })
    }

    /// Accept both versions (left then right)
    pub fn accept_both(&self, block_index: usize, left_first: bool) -> Result<MergeOperation, String> {
        let opcode = self.get_opcode(block_index)?;
        let left_lines = self.left_lines[opcode.left_start..opcode.left_end].to_vec();
        let right_lines = self.right_lines[opcode.right_start..opcode.right_end].to_vec();

        let lines = if left_first {
            [left_lines, right_lines].concat()
        } else {
            [right_lines, left_lines].concat()
        };

        Ok(MergeOperation {
            block_index,
            action: MergeAction::Custom,
            lines,
        })
    }

    /// Apply custom resolution
    pub fn apply_custom(&self, block_index: usize, custom_lines: Vec<String>) -> Result<MergeOperation, String> {
        if block_index >= self.opcodes.len() {
            return Err(format!("Block index {} out of range", block_index));
        }

        Ok(MergeOperation {
            block_index,
            action: MergeAction::Custom,
            lines: custom_lines,
        })
    }

    /// Get opcode for block index
    fn get_opcode(&self, block_index: usize) -> Result<&ThreeWayOpcode, String> {
        self.opcodes
            .get(block_index)
            .ok_or_else(|| format!("Block index {} out of range", block_index))
    }

    /// Get lines for a specific version and block
    pub fn get_block_lines(&self, block_index: usize, version: BlockVersion) -> Result<Vec<String>, String> {
        let opcode = self.get_opcode(block_index)?;

        let lines = match version {
            BlockVersion::Base => self.base_lines[opcode.base_start..opcode.base_end].to_vec(),
            BlockVersion::Left => self.left_lines[opcode.left_start..opcode.left_end].to_vec(),
            BlockVersion::Right => self.right_lines[opcode.right_start..opcode.right_end].to_vec(),
        };

        Ok(lines)
    }

    /// Compare two versions of a block
    pub fn are_versions_equal(&self, block_index: usize, v1: BlockVersion, v2: BlockVersion) -> Result<bool, String> {
        let lines1 = self.get_block_lines(block_index, v1)?;
        let lines2 = self.get_block_lines(block_index, v2)?;
        Ok(lines1 == lines2)
    }
}

/// Version selector for merge operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockVersion {
    Base,
    Left,
    Right,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ConflictType;

    #[test]
    fn test_accept_left() {
        let opcodes = vec![ThreeWayOpcode {
            base_start: 0,
            base_end: 1,
            left_start: 0,
            left_end: 1,
            right_start: 0,
            right_end: 1,
            conflict_type: ConflictType::BothModified,
            auto_mergeable: false,
            resolved_lines: None,
        }];

        let diff = ThreeWayDiffResult {
            opcodes,
            conflicts: vec![],
            auto_merge_possible: false,
        };

        let base = vec!["base line".to_string()];
        let left = vec!["left line".to_string()];
        let right = vec!["right line".to_string()];

        let ops = MergeOperations::new(&diff, &base, &left, &right);
        let result = ops.accept_left(0).unwrap();

        assert_eq!(result.action, MergeAction::AcceptLeft);
        assert_eq!(result.lines, vec!["left line"]);
    }

    #[test]
    fn test_accept_both() {
        let opcodes = vec![ThreeWayOpcode {
            base_start: 0,
            base_end: 1,
            left_start: 0,
            left_end: 1,
            right_start: 0,
            right_end: 1,
            conflict_type: ConflictType::BothModified,
            auto_mergeable: false,
            resolved_lines: None,
        }];

        let diff = ThreeWayDiffResult {
            opcodes,
            conflicts: vec![],
            auto_merge_possible: false,
        };

        let base = vec!["base".to_string()];
        let left = vec!["left".to_string()];
        let right = vec!["right".to_string()];

        let ops = MergeOperations::new(&diff, &base, &left, &right);
        let result = ops.accept_both(0, true).unwrap();

        assert_eq!(result.lines, vec!["left", "right"]);
    }
}
