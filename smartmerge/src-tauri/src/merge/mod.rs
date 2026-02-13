/// Merge engine for SmartMerge
/// 
/// Provides interactive merge capabilities with:
/// - Block-level merge operations (accept left/right/base/both/custom)
/// - Merge state tracking
/// - Conflict resolution with multiple strategies
/// - Undo/redo support
/// - Merge result building

mod state;
mod operations;
mod history;
mod builder;
mod resolver;

pub use state::{MergeAction, MergeState};
pub use operations::MergeOperations;
pub use history::MergeHistory;
pub use builder::{MergeResult, MergeBuilder};
pub use resolver::{ConflictAnalysis, ConflictResolver};

use crate::models::ThreeWayDiffResult;

/// Main merge engine
pub struct MergeEngine {
    /// Current merge state
    state: MergeState,
    /// History for undo/redo
    history: MergeHistory,
    /// Three-way diff result
    diff_result: ThreeWayDiffResult,
    /// Base lines
    base_lines: Vec<String>,
    /// Left lines
    left_lines: Vec<String>,
    /// Right lines
    right_lines: Vec<String>,
}

impl MergeEngine {
    /// Create a new merge engine
    pub fn new(
        diff_result: ThreeWayDiffResult,
        base_lines: Vec<String>,
        left_lines: Vec<String>,
        right_lines: Vec<String>,
    ) -> Self {
        let state = MergeState::from_opcodes(&diff_result.opcodes);
        
        Self {
            state,
            history: MergeHistory::default(),
            diff_result,
            base_lines,
            left_lines,
            right_lines,
        }
    }

    /// Accept left version of a block
    pub fn accept_left(&mut self, block_index: usize) -> Result<(), String> {
        self.apply_action(block_index, MergeAction::AcceptLeft, None, "Accept left version".to_string())
    }

    /// Accept right version of a block
    pub fn accept_right(&mut self, block_index: usize) -> Result<(), String> {
        self.apply_action(block_index, MergeAction::AcceptRight, None, "Accept right version".to_string())
    }

    /// Accept base version of a block
    pub fn accept_base(&mut self, block_index: usize) -> Result<(), String> {
        self.apply_action(block_index, MergeAction::AcceptBase, None, "Accept base version".to_string())
    }

    /// Accept both versions
    pub fn accept_both(&mut self, block_index: usize, left_first: bool) -> Result<(), String> {
        let ops = self.get_operations();
        let operation = ops.accept_both(block_index, left_first)?;
        self.apply_action(block_index, MergeAction::Custom, Some(operation.lines), "Accept both versions".to_string())
    }

    /// Apply custom resolution
    pub fn apply_custom(&mut self, block_index: usize, custom_lines: Vec<String>) -> Result<(), String> {
        self.apply_action(block_index, MergeAction::Custom, Some(custom_lines), "Apply custom resolution".to_string())
    }

    /// Skip a block (leave unresolved for now)
    pub fn skip_block(&mut self, block_index: usize) -> Result<(), String> {
        self.apply_action(block_index, MergeAction::Skip, None, "Skip block".to_string())
    }

    /// Undo last action
    pub fn undo(&mut self) -> Result<(), String> {
        let entry = self.history.undo()
            .ok_or("No actions to undo")?;

        let block_index = entry.block_index;
        let previous_action = entry.previous_action;
        let previous_custom_lines = entry.previous_custom_lines.clone();

        // Don't add to history - we're undoing
        self.state.apply_action(block_index, previous_action, previous_custom_lines)?;
        
        Ok(())
    }

    /// Redo next action
    pub fn redo(&mut self) -> Result<(), String> {
        let entry = self.history.redo()
            .ok_or("No actions to redo")?;

        let block_index = entry.block_index;
        let new_action = entry.new_action;
        let new_custom_lines = entry.new_custom_lines.clone();

        // Don't add to history - we're redoing
        self.state.apply_action(block_index, new_action, new_custom_lines)?;
        
        Ok(())
    }

    /// Auto-resolve all trivial conflicts
    pub fn auto_resolve_all(&mut self) -> Result<usize, String> {
        let ops = self.get_operations();
        let resolver = ConflictResolver::new(&ops);
        
        let resolutions = resolver.batch_auto_resolve(&self.diff_result.opcodes);
        let count = resolutions.len();

        for (block_index, action) in resolutions {
            self.apply_action(block_index, action, None, "Auto-resolved".to_string())?;
        }

        Ok(count)
    }

    /// Build final merge result
    pub fn build_result(&self) -> Result<MergeResult, String> {
        let builder = MergeBuilder::new(
            &self.state,
            &self.diff_result,
            &self.base_lines,
            &self.left_lines,
            &self.right_lines,
        );
        builder.build()
    }

    /// Build partial result (only resolved blocks)
    pub fn build_partial_result(&self) -> Result<MergeResult, String> {
        let builder = MergeBuilder::new(
            &self.state,
            &self.diff_result,
            &self.base_lines,
            &self.left_lines,
            &self.right_lines,
        );
        builder.build_partial()
    }

    /// Get current merge state
    pub fn state(&self) -> &MergeState {
        &self.state
    }

    /// Get operations handler
    fn get_operations(&self) -> MergeOperations<'_> {
        MergeOperations::new(&self.diff_result, &self.base_lines, &self.left_lines, &self.right_lines)
    }

    /// Apply an action with history tracking
    fn apply_action(
        &mut self,
        block_index: usize,
        new_action: MergeAction,
        new_custom_lines: Option<Vec<String>>,
        description: String,
    ) -> Result<(), String> {
        // Save previous state
        let block = &self.state.blocks[block_index];
        let previous_action = block.action;
        let previous_custom_lines = block.custom_lines.clone();

        // Apply new action
        self.state.apply_action(block_index, new_action, new_custom_lines.clone())?;

        // Record in history
        self.history.push(
            block_index,
            previous_action,
            previous_custom_lines,
            new_action,
            new_custom_lines,
            description,
        );

        Ok(())
    }

    /// Get unresolved conflicts
    pub fn get_unresolved_conflicts(&self) -> Vec<usize> {
        self.state.get_unresolved_conflicts()
    }

    /// Check if merge is complete
    pub fn is_complete(&self) -> bool {
        self.state.is_complete()
    }

    /// Get completion percentage
    pub fn completion_percentage(&self) -> f32 {
        self.state.completion_percentage()
    }

    /// Can undo
    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    /// Can redo
    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    /// Analyze a specific conflict
    pub fn analyze_conflict(&self, block_index: usize) -> Result<ConflictAnalysis, String> {
        let ops = self.get_operations();
        let resolver = ConflictResolver::new(&ops);
        let opcode = self.diff_result.opcodes.get(block_index)
            .ok_or("Block index out of range")?;
        Ok(resolver.analyze_conflict(block_index, opcode))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ConflictType;

    #[test]
    fn test_merge_engine_creation() {
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

        let engine = MergeEngine::new(diff, base, left, right);
        assert!(!engine.is_complete());
    }

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

        let base = vec!["base".to_string()];
        let left = vec!["left".to_string()];
        let right = vec!["right".to_string()];

        let mut engine = MergeEngine::new(diff, base, left, right);
        engine.accept_left(0).unwrap();
        
        assert!(engine.is_complete());
        assert!(engine.can_undo());
    }
}
