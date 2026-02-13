use crate::models::{ThreeWayOpcode, ConflictType};
use super::operations::{MergeOperations, BlockVersion};
use super::state::MergeAction;

/// Conflict resolution strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolutionStrategy {
    /// Always accept left (ours)
    PreferLeft,
    /// Always accept right (theirs)
    PreferRight,
    /// Accept newer version (based on timestamps, if available)
    PreferNewer,
    /// Accept longer version
    PreferLonger,
    /// Accept shorter version
    PreferShorter,
    /// Accept both (left then right)
    AcceptBoth,
    /// Manual resolution required
    Manual,
}

/// Conflict resolver
pub struct ConflictResolver<'a> {
    operations: &'a MergeOperations<'a>,
}

impl<'a> ConflictResolver<'a> {
    pub fn new(operations: &'a MergeOperations<'a>) -> Self {
        Self { operations }
    }

    /// Suggest resolution for a conflict block
    pub fn suggest_resolution(&self, block_index: usize, strategy: ResolutionStrategy) -> Result<MergeAction, String> {
        match strategy {
            ResolutionStrategy::PreferLeft => Ok(MergeAction::AcceptLeft),
            ResolutionStrategy::PreferRight => Ok(MergeAction::AcceptRight),
            ResolutionStrategy::PreferNewer => {
                // Without timestamp metadata, default to left
                Ok(MergeAction::AcceptLeft)
            }
            ResolutionStrategy::PreferLonger => self.prefer_longer(block_index),
            ResolutionStrategy::PreferShorter => self.prefer_shorter(block_index),
            ResolutionStrategy::AcceptBoth => Ok(MergeAction::Custom),
            ResolutionStrategy::Manual => Ok(MergeAction::Pending),
        }
    }

    /// Automatically resolve a conflict if possible
    pub fn auto_resolve(&self, block_index: usize, opcode: &ThreeWayOpcode) -> Option<MergeAction> {
        // If both sides made identical changes, auto-resolve to either
        if self.operations.are_versions_equal(block_index, BlockVersion::Left, BlockVersion::Right).ok()? {
            return Some(MergeAction::AcceptLeft);
        }

        // If left == base, accept right (only right changed)
        if self.operations.are_versions_equal(block_index, BlockVersion::Left, BlockVersion::Base).ok()? {
            return Some(MergeAction::AcceptRight);
        }

        // If right == base, accept left (only left changed)
        if self.operations.are_versions_equal(block_index, BlockVersion::Right, BlockVersion::Base).ok()? {
            return Some(MergeAction::AcceptLeft);
        }

        // Complex conflict - needs manual resolution
        None
    }

    /// Analyze conflict complexity
    pub fn analyze_conflict(&self, block_index: usize, opcode: &ThreeWayOpcode) -> ConflictAnalysis {
        let left_lines = self.operations.get_block_lines(block_index, BlockVersion::Left).unwrap_or_default();
        let right_lines = self.operations.get_block_lines(block_index, BlockVersion::Right).unwrap_or_default();
        let base_lines = self.operations.get_block_lines(block_index, BlockVersion::Base).unwrap_or_default();

        let left_same_as_base = left_lines == base_lines;
        let right_same_as_base = right_lines == base_lines;
        let left_same_as_right = left_lines == right_lines;

        let complexity = if left_same_as_right {
            ConflictComplexity::Trivial
        } else if left_same_as_base || right_same_as_base {
            ConflictComplexity::Simple
        } else {
            ConflictComplexity::Complex
        };

        ConflictAnalysis {
            conflict_type: opcode.conflict_type.clone(),
            complexity,
            left_line_count: left_lines.len(),
            right_line_count: right_lines.len(),
            base_line_count: base_lines.len(),
            auto_resolvable: self.auto_resolve(block_index, opcode).is_some(),
            suggested_action: self.suggest_resolution(block_index, ResolutionStrategy::PreferLeft).ok(),
        }
    }

    fn prefer_longer(&self, block_index: usize) -> Result<MergeAction, String> {
        let left_lines = self.operations.get_block_lines(block_index, BlockVersion::Left)?;
        let right_lines = self.operations.get_block_lines(block_index, BlockVersion::Right)?;

        if left_lines.len() >= right_lines.len() {
            Ok(MergeAction::AcceptLeft)
        } else {
            Ok(MergeAction::AcceptRight)
        }
    }

    fn prefer_shorter(&self, block_index: usize) -> Result<MergeAction, String> {
        let left_lines = self.operations.get_block_lines(block_index, BlockVersion::Left)?;
        let right_lines = self.operations.get_block_lines(block_index, BlockVersion::Right)?;

        if left_lines.len() <= right_lines.len() {
            Ok(MergeAction::AcceptLeft)
        } else {
            Ok(MergeAction::AcceptRight)
        }
    }

    /// Batch resolve all auto-resolvable conflicts
    pub fn batch_auto_resolve(&self, opcodes: &[ThreeWayOpcode]) -> Vec<(usize, MergeAction)> {
        opcodes
            .iter()
            .enumerate()
            .filter_map(|(index, opcode)| {
                if opcode.conflict_type != ConflictType::None {
                    self.auto_resolve(index, opcode).map(|action| (index, action))
                } else {
                    None
                }
            })
            .collect()
    }
}

/// Conflict complexity level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictComplexity {
    /// Both sides made identical changes
    Trivial,
    /// Only one side changed
    Simple,
    /// Both sides changed differently
    Complex,
}

/// Conflict analysis result
#[derive(Debug, Clone)]
pub struct ConflictAnalysis {
    pub conflict_type: ConflictType,
    pub complexity: ConflictComplexity,
    pub left_line_count: usize,
    pub right_line_count: usize,
    pub base_line_count: usize,
    pub auto_resolvable: bool,
    pub suggested_action: Option<MergeAction>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ThreeWayDiffResult;

    #[test]
    fn test_prefer_longer() {
        let opcodes = vec![ThreeWayOpcode {
            base_start: 0,
            base_end: 1,
            left_start: 0,
            left_end: 2,
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

        let base = vec!["line 1".to_string()];
        let left = vec!["line 1".to_string(), "line 2".to_string()];
        let right = vec!["line 1".to_string()];

        let ops = MergeOperations::new(&diff, &base, &left, &right);
        let resolver = ConflictResolver::new(&ops);

        let action = resolver.prefer_longer(0).unwrap();
        assert_eq!(action, MergeAction::AcceptLeft);
    }

    #[test]
    fn test_resolution_strategy() {
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
        let resolver = ConflictResolver::new(&ops);

        let action = resolver.suggest_resolution(0, ResolutionStrategy::PreferRight).unwrap();
        assert_eq!(action, MergeAction::AcceptRight);
    }
}
