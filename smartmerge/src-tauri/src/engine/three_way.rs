use crate::models::{ThreeWayOpcode, ThreeWayDiffResult, ConflictType, Opcode};
use super::api::DiffEngine;

/// Three-way diff for merge operations
pub struct ThreeWayDiff {
    engine: DiffEngine,
}

impl ThreeWayDiff {
    pub fn new(engine: DiffEngine) -> Self {
        Self { engine }
    }

    /// Perform three-way diff between base, left (ours), and right (theirs)
    pub fn diff(
        &self,
        base: &[String],
        left: &[String],
        right: &[String],
    ) -> ThreeWayDiffResult {
        // Diff base against left (ours)
        let left_opcodes = self.engine.compute_diff(base, left);
        
        // Diff base against right (theirs)
        let right_opcodes = self.engine.compute_diff(base, right);
        
        // Merge the two diffs
        let three_way_opcodes = self.merge_diffs(base, left, right, &left_opcodes, &right_opcodes);
        
        ThreeWayDiffResult::new(
            base.to_vec(),
            left.to_vec(),
            right.to_vec(),
            three_way_opcodes,
        )
    }

    /// Merge two two-way diffs into a three-way diff
    fn merge_diffs(
        &self,
        base: &[String],
        left: &[String],
        right: &[String],
        left_ops: &[Opcode],
        right_ops: &[Opcode],
    ) -> Vec<ThreeWayOpcode> {
        let mut result = Vec::new();
        let mut base_pos = 0;
        let mut left_pos = 0;
        let mut right_pos = 0;
        
        let mut left_idx = 0;
        let mut right_idx = 0;
        
        while left_idx < left_ops.len() || right_idx < right_ops.len() {
            let left_op = left_ops.get(left_idx);
            let right_op = right_ops.get(right_idx);
            
            match (left_op, right_op) {
                (Some(l_op), Some(r_op)) => {
                    // Check if operations overlap
                    if l_op.i2 <= r_op.i1 {
                        // Left operation comes first
                        let three_way = self.process_left_change(
                            l_op, base_pos, left_pos, right_pos, base, left, right
                        );
                        result.push(three_way);
                        base_pos = l_op.i2;
                        left_pos = l_op.j2;
                        left_idx += 1;
                    } else if r_op.i2 <= l_op.i1 {
                        // Right operation comes first
                        let three_way = self.process_right_change(
                            r_op, base_pos, left_pos, right_pos, base, left, right
                        );
                        result.push(three_way);
                        base_pos = r_op.i2;
                        right_pos = r_op.j2;
                        right_idx += 1;
                    } else {
                        // Operations overlap - potential conflict
                        let three_way = self.process_conflict(
                            l_op, r_op, base_pos, left_pos, right_pos, base, left, right
                        );
                        result.push(three_way);
                        base_pos = l_op.i2.max(r_op.i2);
                        left_pos = l_op.j2;
                        right_pos = r_op.j2;
                        left_idx += 1;
                        right_idx += 1;
                    }
                }
                (Some(l_op), None) => {
                    // Only left changes remain
                    let three_way = self.process_left_change(
                        l_op, base_pos, left_pos, right_pos, base, left, right
                    );
                    result.push(three_way);
                    base_pos = l_op.i2;
                    left_pos = l_op.j2;
                    left_idx += 1;
                }
                (None, Some(r_op)) => {
                    // Only right changes remain
                    let three_way = self.process_right_change(
                        r_op, base_pos, left_pos, right_pos, base, left, right
                    );
                    result.push(three_way);
                    base_pos = r_op.i2;
                    right_pos = r_op.j2;
                    right_idx += 1;
                }
                (None, None) => break,
            }
        }
        
        result
    }

    fn process_left_change(
        &self,
        op: &Opcode,
        _base_pos: usize,
        left_pos: usize,
        right_pos: usize,
        _base: &[String],
        left: &[String],
        _right: &[String],
    ) -> ThreeWayOpcode {
        // Left changed, right unchanged - auto-mergeable
        let resolved = if op.tag == "equal" {
            None
        } else {
            Some(left[left_pos..left_pos + (op.j2 - op.j1)].to_vec())
        };
        
        ThreeWayOpcode {
            base_start: op.i1,
            base_end: op.i2,
            left_start: op.j1,
            left_end: op.j2,
            right_start: right_pos,
            right_end: right_pos + (op.i2 - op.i1),
            conflict_type: ConflictType::None,
            auto_mergeable: true,
            resolved_lines: resolved,
        }
    }

    fn process_right_change(
        &self,
        op: &Opcode,
        _base_pos: usize,
        left_pos: usize,
        _right_pos: usize,
        _base: &[String],
        _left: &[String],
        right: &[String],
    ) -> ThreeWayOpcode {
        // Right changed, left unchanged - auto-mergeable
        let resolved = if op.tag == "equal" {
            None
        } else {
            Some(right[op.j1..op.j2].to_vec())
        };
        
        ThreeWayOpcode {
            base_start: op.i1,
            base_end: op.i2,
            left_start: left_pos,
            left_end: left_pos + (op.i2 - op.i1),
            right_start: op.j1,
            right_end: op.j2,
            conflict_type: ConflictType::None,
            auto_mergeable: true,
            resolved_lines: resolved,
        }
    }

    fn process_conflict(
        &self,
        left_op: &Opcode,
        right_op: &Opcode,
        _base_pos: usize,
        _left_pos: usize,
        _right_pos: usize,
        _base: &[String],
        left: &[String],
        right: &[String],
    ) -> ThreeWayOpcode {
        // Both sides changed - check if changes are identical
        let left_lines = &left[left_op.j1..left_op.j2];
        let right_lines = &right[right_op.j1..right_op.j2];
        
        let conflict_type;
        let auto_mergeable;
        let resolved;
        
        if left_lines == right_lines {
            // Both sides made identical changes - auto-merge
            conflict_type = ConflictType::None;
            auto_mergeable = true;
            resolved = Some(left_lines.to_vec());
        } else {
            // Different changes - conflict
            conflict_type = if left_op.tag == "delete" && right_op.tag != "delete" {
                ConflictType::DeleteModify
            } else if left_op.tag != "delete" && right_op.tag == "delete" {
                ConflictType::ModifyDelete
            } else if left_op.tag == "insert" && right_op.tag == "insert" {
                ConflictType::BothAdded
            } else {
                ConflictType::BothModified
            };
            auto_mergeable = false;
            resolved = None;
        }
        
        ThreeWayOpcode {
            base_start: left_op.i1.min(right_op.i1),
            base_end: left_op.i2.max(right_op.i2),
            left_start: left_op.j1,
            left_end: left_op.j2,
            right_start: right_op.j1,
            right_end: right_op.j2,
            conflict_type,
            auto_mergeable,
            resolved_lines: resolved,
        }
    }
}

/// Auto-merge three-way diff when possible
pub fn auto_merge(result: &mut ThreeWayDiffResult) -> Result<Vec<String>, String> {
    if result.has_conflicts() {
        return Err(format!("Cannot auto-merge: {} conflicts remain", result.conflict_count()));
    }
    
    let mut merged = Vec::new();
    let mut base_pos = 0;
    
    for opcode in &result.opcodes {
        // Add unchanged lines from base
        if opcode.base_start > base_pos {
            merged.extend_from_slice(&result.base_lines[base_pos..opcode.base_start]);
        }
        
        // Add resolved lines or determine which side to take
        if let Some(ref resolved) = opcode.resolved_lines {
            merged.extend_from_slice(resolved);
        } else {
            // Take left if it changed, otherwise right, otherwise base
            let left_changed = opcode.left_end > opcode.left_start;
            let right_changed = opcode.right_end > opcode.right_start;
            
            if left_changed {
                merged.extend_from_slice(&result.left_lines[opcode.left_start..opcode.left_end]);
            } else if right_changed {
                merged.extend_from_slice(&result.right_lines[opcode.right_start..opcode.right_end]);
            } else if opcode.base_end > opcode.base_start {
                merged.extend_from_slice(&result.base_lines[opcode.base_start..opcode.base_end]);
            }
        }
        
        base_pos = opcode.base_end;
    }
    
    // Add any remaining base lines
    if base_pos < result.base_lines.len() {
        merged.extend_from_slice(&result.base_lines[base_pos..]);
    }
    
    result.merged_lines = Some(merged.clone());
    Ok(merged)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::AlgorithmType;
    
    #[test]
    fn test_three_way_no_conflict() {
        let engine = DiffEngine::with_algorithm(AlgorithmType::Myers);
        let three_way = ThreeWayDiff::new(engine);
        
        let base = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let left = vec!["a".to_string(), "b_modified".to_string(), "c".to_string()];
        let right = vec!["a".to_string(), "b".to_string(), "c_modified".to_string()];
        
        let result = three_way.diff(&base, &left, &right);
        
        assert!(!result.has_conflicts());
    }
    
    #[test]
    fn test_three_way_with_conflict() {
        let engine = DiffEngine::with_algorithm(AlgorithmType::Myers);
        let three_way = ThreeWayDiff::new(engine);
        
        let base = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let left = vec!["a".to_string(), "b_left".to_string(), "c".to_string()];
        let right = vec!["a".to_string(), "b_right".to_string(), "c".to_string()];
        
        let result = three_way.diff(&base, &left, &right);
        
        assert!(result.has_conflicts());
        assert_eq!(result.conflict_count(), 1);
    }
    
    #[test]
    fn test_three_way_identical_changes() {
        let engine = DiffEngine::with_algorithm(AlgorithmType::Myers);
        let three_way = ThreeWayDiff::new(engine);
        
        let base = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let left = vec!["a".to_string(), "b_modified".to_string(), "c".to_string()];
        let right = vec!["a".to_string(), "b_modified".to_string(), "c".to_string()];
        
        let result = three_way.diff(&base, &left, &right);
        
        assert!(!result.has_conflicts());
    }
}
