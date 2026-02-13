use crate::models::Opcode;
use serde::{Serialize, Deserialize};

/// Represents a moved block of lines
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MovedBlock {
    /// Start line in left file
    pub left_start: usize,
    /// End line in left file (exclusive)
    pub left_end: usize,
    /// Start line in right file
    pub right_start: usize,
    /// End line in right file (exclusive)
    pub right_end: usize,
    /// The lines that were moved
    pub lines: Vec<String>,
}

/// Detect moved blocks of code between two files
/// Returns a list of moved blocks found
pub fn detect_moved_blocks(
    left: &[String],
    right: &[String],
    opcodes: &[Opcode],
    min_block_size: usize,
) -> Vec<MovedBlock> {
    let mut moved_blocks = Vec::new();
    
    // Find deleted blocks in left and inserted blocks in right
    let deleted_blocks = find_deleted_blocks(left, opcodes, min_block_size);
    let inserted_blocks = find_inserted_blocks(right, opcodes, min_block_size);
    
    // Match deleted and inserted blocks
    for deleted in &deleted_blocks {
        for inserted in &inserted_blocks {
            if blocks_match(&deleted.lines, &inserted.lines) {
                moved_blocks.push(MovedBlock {
                    left_start: deleted.left_start,
                    left_end: deleted.left_end,
                    right_start: inserted.right_start,
                    right_end: inserted.right_end,
                    lines: deleted.lines.clone(),
                });
            }
        }
    }
    
    // Remove overlaps - keep the largest blocks
    deduplicate_moved_blocks(&mut moved_blocks);
    
    moved_blocks
}

#[derive(Debug, Clone)]
struct Block {
    left_start: usize,
    left_end: usize,
    right_start: usize,
    right_end: usize,
    lines: Vec<String>,
}

fn find_deleted_blocks(left: &[String], opcodes: &[Opcode], min_size: usize) -> Vec<Block> {
    let mut blocks = Vec::new();
    
    for opcode in opcodes {
        if opcode.tag == "delete" {
            let size = opcode.i2 - opcode.i1;
            if size >= min_size {
                let lines = left[opcode.i1..opcode.i2].to_vec();
                blocks.push(Block {
                    left_start: opcode.i1,
                    left_end: opcode.i2,
                    right_start: opcode.j1,
                    right_end: opcode.j1,
                    lines,
                });
            }
        } else if opcode.tag == "replace" {
            let left_size = opcode.i2 - opcode.i1;
            if left_size >= min_size {
                let lines = left[opcode.i1..opcode.i2].to_vec();
                blocks.push(Block {
                    left_start: opcode.i1,
                    left_end: opcode.i2,
                    right_start: opcode.j1,
                    right_end: opcode.j1,
                    lines,
                });
            }
        }
    }
    
    blocks
}

fn find_inserted_blocks(right: &[String], opcodes: &[Opcode], min_size: usize) -> Vec<Block> {
    let mut blocks = Vec::new();
    
    for opcode in opcodes {
        if opcode.tag == "insert" {
            let size = opcode.j2 - opcode.j1;
            if size >= min_size {
                let lines = right[opcode.j1..opcode.j2].to_vec();
                blocks.push(Block {
                    left_start: opcode.i1,
                    left_end: opcode.i1,
                    right_start: opcode.j1,
                    right_end: opcode.j2,
                    lines,
                });
            }
        } else if opcode.tag == "replace" {
            let right_size = opcode.j2 - opcode.j1;
            if right_size >= min_size {
                let lines = right[opcode.j1..opcode.j2].to_vec();
                blocks.push(Block {
                    left_start: opcode.i1,
                    left_end: opcode.i1,
                    right_start: opcode.j1,
                    right_end: opcode.j2,
                    lines,
                });
            }
        }
    }
    
    blocks
}

fn blocks_match(left_lines: &[String], right_lines: &[String]) -> bool {
    if left_lines.len() != right_lines.len() {
        return false;
    }
    
    // Exact match
    if left_lines == right_lines {
        return true;
    }
    
    // Allow for minor whitespace differences
    let threshold = (left_lines.len() as f32 * 0.9) as usize;
    let mut matches = 0;
    
    for (left, right) in left_lines.iter().zip(right_lines.iter()) {
        if left.trim() == right.trim() {
            matches += 1;
        }
    }
    
    matches >= threshold
}

fn deduplicate_moved_blocks(blocks: &mut Vec<MovedBlock>) {
    if blocks.len() <= 1 {
        return;
    }
    
    // Sort by size (largest first)
    blocks.sort_by(|a, b| {
        let size_a = a.left_end - a.left_start;
        let size_b = b.left_end - b.left_start;
        size_b.cmp(&size_a)
    });
    
    // Remove overlapping blocks
    let mut kept = Vec::new();
    for block in blocks.iter() {
        let mut overlaps = false;
        for kept_block in &kept {
            if blocks_overlap(block, kept_block) {
                overlaps = true;
                break;
            }
        }
        if !overlaps {
            kept.push(block.clone());
        }
    }
    
    *blocks = kept;
}

fn blocks_overlap(a: &MovedBlock, b: &MovedBlock) -> bool {
    // Check left side overlap
    let left_overlap = !(a.left_end <= b.left_start || b.left_end <= a.left_start);
    
    // Check right side overlap
    let right_overlap = !(a.right_end <= b.right_start || b.right_end <= a.right_start);
    
    left_overlap || right_overlap
}

/// Create a summary of moved blocks for display
pub fn summarize_moved_blocks(blocks: &[MovedBlock]) -> String {
    if blocks.is_empty() {
        return "No moved blocks detected".to_string();
    }
    
    let mut summary = format!("Found {} moved block(s):\n", blocks.len());
    for (i, block) in blocks.iter().enumerate() {
        let size = block.left_end - block.left_start;
        summary.push_str(&format!(
            "  {}. Lines {}-{} moved to lines {}-{} ({} lines)\n",
            i + 1,
            block.left_start + 1,
            block.left_end,
            block.right_start + 1,
            block.right_end,
            size
        ));
    }
    summary
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_blocks_match_exact() {
        let left = vec!["a".to_string(), "b".to_string()];
        let right = vec!["a".to_string(), "b".to_string()];
        assert!(blocks_match(&left, &right));
    }
    
    #[test]
    fn test_blocks_match_whitespace() {
        let left = vec!["a".to_string(), "b".to_string()];
        let right = vec!["a  ".to_string(), "  b".to_string()];
        assert!(blocks_match(&left, &right));
    }
    
    #[test]
    fn test_blocks_dont_match() {
        let left = vec!["a".to_string(), "b".to_string()];
        let right = vec!["x".to_string(), "y".to_string()];
        assert!(!blocks_match(&left, &right));
    }
    
    #[test]
    fn test_detect_moved_blocks_simple() {
        let left = vec![
            "line1".to_string(),
            "block_a".to_string(),
            "block_b".to_string(),
            "line2".to_string(),
        ];
        let right = vec![
            "line1".to_string(),
            "line2".to_string(),
            "block_a".to_string(),
            "block_b".to_string(),
        ];
        
        let opcodes = vec![
            Opcode::new("equal", 0, 1, 0, 1),
            Opcode::new("delete", 1, 3, 1, 1),
            Opcode::new("equal", 3, 4, 1, 2),
            Opcode::new("insert", 4, 4, 2, 4),
        ];
        
        let moved = detect_moved_blocks(&left, &right, &opcodes, 2);
        assert_eq!(moved.len(), 1);
        assert_eq!(moved[0].left_start, 1);
        assert_eq!(moved[0].left_end, 3);
        assert_eq!(moved[0].right_start, 2);
        assert_eq!(moved[0].right_end, 4);
    }
    
    #[test]
    fn test_blocks_overlap() {
        let a = MovedBlock {
            left_start: 0,
            left_end: 5,
            right_start: 10,
            right_end: 15,
            lines: vec![],
        };
        let b = MovedBlock {
            left_start: 3,
            left_end: 8,
            right_start: 20,
            right_end: 25,
            lines: vec![],
        };
        assert!(blocks_overlap(&a, &b));
    }
}
