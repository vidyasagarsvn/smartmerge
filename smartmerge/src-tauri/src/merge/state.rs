use serde::{Serialize, Deserialize};
use std::collections::HashSet;
use crate::models::{ThreeWayOpcode, ConflictType};

/// Action taken on a merge block
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MergeAction {
    /// Block not yet processed
    Pending,
    /// Accept left (ours) version
    AcceptLeft,
    /// Accept right (theirs) version
    AcceptRight,
    /// Accept base (ancestor) version
    AcceptBase,
    /// Custom resolution provided
    Custom,
    /// Block skipped (for now)
    Skip,
    /// Auto-merged successfully
    AutoMerged,
}

/// Status of a merge block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockStatus {
    /// Index of the block (opcode index)
    pub block_index: usize,
    /// Action taken on this block
    pub action: MergeAction,
    /// Whether this block has a conflict
    pub has_conflict: bool,
    /// Conflict type if any
    pub conflict_type: ConflictType,
    /// Custom resolution lines (if action is Custom)
    pub custom_lines: Option<Vec<String>>,
    /// Timestamp when action was taken
    pub timestamp: Option<u64>,
}

impl BlockStatus {
    pub fn new(block_index: usize, conflict_type: ConflictType) -> Self {
        let has_conflict = conflict_type != ConflictType::None;
        let action = if !has_conflict {
            MergeAction::AutoMerged
        } else {
            MergeAction::Pending
        };

        Self {
            block_index,
            action,
            has_conflict,
            conflict_type,
            custom_lines: None,
            timestamp: None,
        }
    }

    pub fn is_resolved(&self) -> bool {
        self.action != MergeAction::Pending && self.action != MergeAction::Skip
    }
}

/// Complete merge state tracker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeState {
    /// Status for each block
    pub blocks: Vec<BlockStatus>,
    /// Indices of unresolved conflicts
    pub unresolved_conflicts: HashSet<usize>,
    /// Indices of resolved conflicts
    pub resolved_conflicts: HashSet<usize>,
    /// Statistics
    pub stats: MergeStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeStats {
    pub total_blocks: usize,
    pub conflict_blocks: usize,
    pub resolved_blocks: usize,
    pub auto_merged_blocks: usize,
    pub pending_blocks: usize,
}

impl MergeState {
    /// Create a new merge state from three-way opcodes
    pub fn from_opcodes(opcodes: &[ThreeWayOpcode]) -> Self {
        let mut blocks = Vec::new();
        let mut unresolved_conflicts = HashSet::new();
        let mut resolved_conflicts = HashSet::new();

        for (index, opcode) in opcodes.iter().enumerate() {
            let status = BlockStatus::new(index, opcode.conflict_type.clone());
            
            if status.has_conflict {
                if opcode.auto_mergeable {
                    resolved_conflicts.insert(index);
                } else {
                    unresolved_conflicts.insert(index);
                }
            }
            
            blocks.push(status);
        }

        let stats = MergeStats {
            total_blocks: blocks.len(),
            conflict_blocks: unresolved_conflicts.len() + resolved_conflicts.len(),
            resolved_blocks: resolved_conflicts.len(),
            auto_merged_blocks: blocks.iter().filter(|b| b.action == MergeAction::AutoMerged).count(),
            pending_blocks: unresolved_conflicts.len(),
        };

        Self {
            blocks,
            unresolved_conflicts,
            resolved_conflicts,
            stats,
        }
    }

    /// Apply an action to a block
    pub fn apply_action(&mut self, block_index: usize, action: MergeAction, custom_lines: Option<Vec<String>>) -> Result<(), String> {
        if block_index >= self.blocks.len() {
            return Err(format!("Block index {} out of range", block_index));
        }

        let block = &mut self.blocks[block_index];
        block.action = action;
        block.custom_lines = custom_lines;
        block.timestamp = Some(current_timestamp());

        // Update conflict tracking
        if block.has_conflict {
            self.unresolved_conflicts.remove(&block_index);
            self.resolved_conflicts.insert(block_index);
        }

        self.update_stats();
        Ok(())
    }

    /// Get all unresolved conflict indices
    pub fn get_unresolved_conflicts(&self) -> Vec<usize> {
        let mut conflicts: Vec<usize> = self.unresolved_conflicts.iter().copied().collect();
        conflicts.sort();
        conflicts
    }

    /// Check if merge is complete
    pub fn is_complete(&self) -> bool {
        self.unresolved_conflicts.is_empty()
    }

    /// Get completion percentage
    pub fn completion_percentage(&self) -> f32 {
        if self.stats.total_blocks == 0 {
            return 100.0;
        }
        (self.stats.resolved_blocks as f32 / self.stats.total_blocks as f32) * 100.0
    }

    fn update_stats(&mut self) {
        self.stats.resolved_blocks = self.blocks.iter().filter(|b| b.is_resolved()).count();
        self.stats.pending_blocks = self.unresolved_conflicts.len();
        self.stats.auto_merged_blocks = self.blocks.iter().filter(|b| b.action == MergeAction::AutoMerged).count();
    }
}

fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_status_creation() {
        let status = BlockStatus::new(0, ConflictType::BothModified);
        assert!(status.has_conflict);
        assert_eq!(status.action, MergeAction::Pending);
    }

    #[test]
    fn test_merge_state_tracking() {
        let opcodes = vec![
            ThreeWayOpcode {
                base_start: 0,
                base_end: 1,
                left_start: 0,
                left_end: 1,
                right_start: 0,
                right_end: 1,
                conflict_type: ConflictType::None,
                auto_mergeable: true,
                resolved_lines: None,
            },
        ];

        let state = MergeState::from_opcodes(&opcodes);
        assert_eq!(state.blocks.len(), 1);
        assert!(state.is_complete());
    }
}
