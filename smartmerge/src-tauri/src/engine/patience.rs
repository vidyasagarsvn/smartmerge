use std::collections::HashMap;
use crate::models::Opcode;
use super::algorithm::DiffAlgorithm;

/// Patience diff algorithm implementation
/// Based on Bram Cohen's patience diff algorithm
/// Produces more intuitive diffs by finding unique common lines as anchors
pub struct PatienceAlgorithm;

impl PatienceAlgorithm {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PatienceAlgorithm {
    fn default() -> Self {
        Self::new()
    }
}

impl DiffAlgorithm for PatienceAlgorithm {
    fn compute_diff(&self, left: &[String], right: &[String]) -> Vec<Opcode> {
        patience_diff(left, right, 0, 0)
    }
    
    fn name(&self) -> &str {
        "Patience"
    }
    
    fn description(&self) -> &str {
        "Patience diff algorithm - produces more intuitive diffs using unique line matching"
    }
}

fn patience_diff(left: &[String], right: &[String], left_offset: usize, right_offset: usize) -> Vec<Opcode> {
    if left.is_empty() && right.is_empty() {
        return Vec::new();
    }
    
    if left.is_empty() {
        return vec![Opcode::new("insert", left_offset, left_offset, right_offset, right_offset + right.len())];
    }
    
    if right.is_empty() {
        return vec![Opcode::new("delete", left_offset, left_offset + left.len(), right_offset, right_offset)];
    }
    
    // Find unique lines in both sequences
    let unique_common = find_unique_common_lines(left, right);
    
    if unique_common.is_empty() {
        // No unique common lines, fall back to simple diff
        return simple_diff(left, right, left_offset, right_offset);
    }
    
    // Use LCS on unique common lines to find longest increasing subsequence
    let lcs = longest_increasing_subsequence(&unique_common);
    
    // Build opcodes recursively using LCS as anchors
    let mut opcodes = Vec::new();
    let mut left_pos = 0;
    let mut right_pos = 0;
    
    for &(left_idx, right_idx) in &lcs {
        // Recursively diff regions before this anchor
        if left_pos < left_idx || right_pos < right_idx {
            let left_slice = &left[left_pos..left_idx];
            let right_slice = &right[right_pos..right_idx];
            let sub_opcodes = patience_diff(
                left_slice,
                right_slice,
                left_offset + left_pos,
                right_offset + right_pos,
            );
            opcodes.extend(sub_opcodes);
        }
        
        // Add the matching line
        opcodes.push(Opcode::new(
            "equal",
            left_offset + left_idx,
            left_offset + left_idx + 1,
            right_offset + right_idx,
            right_offset + right_idx + 1,
        ));
        
        left_pos = left_idx + 1;
        right_pos = right_idx + 1;
    }
    
    // Handle remaining lines after last anchor
    if left_pos < left.len() || right_pos < right.len() {
        let left_slice = &left[left_pos..];
        let right_slice = &right[right_pos..];
        let sub_opcodes = patience_diff(
            left_slice,
            right_slice,
            left_offset + left_pos,
            right_offset + right_pos,
        );
        opcodes.extend(sub_opcodes);
    }
    
    merge_adjacent_opcodes(&opcodes)
}

/// Find lines that appear exactly once in both sequences
fn find_unique_common_lines(left: &[String], right: &[String]) -> Vec<(usize, usize)> {
    let mut left_counts: HashMap<&String, Vec<usize>> = HashMap::new();
    let mut right_counts: HashMap<&String, Vec<usize>> = HashMap::new();
    
    for (i, line) in left.iter().enumerate() {
        left_counts.entry(line).or_insert_with(Vec::new).push(i);
    }
    
    for (i, line) in right.iter().enumerate() {
        right_counts.entry(line).or_insert_with(Vec::new).push(i);
    }
    
    let mut unique_common = Vec::new();
    for (line, left_indices) in &left_counts {
        if left_indices.len() == 1 {
            if let Some(right_indices) = right_counts.get(line) {
                if right_indices.len() == 1 {
                    unique_common.push((left_indices[0], right_indices[0]));
                }
            }
        }
    }
    
    // Sort by left index
    unique_common.sort_by_key(|&(l, _)| l);
    unique_common
}

/// Find longest increasing subsequence of right indices
fn longest_increasing_subsequence(pairs: &[(usize, usize)]) -> Vec<(usize, usize)> {
    if pairs.is_empty() {
        return Vec::new();
    }
    
    let n = pairs.len();
    let mut dp = vec![1; n];
    let mut parent = vec![None; n];
    
    for i in 1..n {
        for j in 0..i {
            if pairs[j].1 < pairs[i].1 && dp[j] + 1 > dp[i] {
                dp[i] = dp[j] + 1;
                parent[i] = Some(j);
            }
        }
    }
    
    // Find the index with maximum length
    let mut max_idx = 0;
    for i in 1..n {
        if dp[i] > dp[max_idx] {
            max_idx = i;
        }
    }
    
    // Reconstruct the LCS
    let mut result = Vec::new();
    let mut idx = Some(max_idx);
    while let Some(i) = idx {
        result.push(pairs[i]);
        idx = parent[i];
    }
    
    result.reverse();
    result
}

/// Simple diff fallback for regions without unique common lines
fn simple_diff(left: &[String], right: &[String], left_offset: usize, right_offset: usize) -> Vec<Opcode> {
    let mut opcodes = Vec::new();
    let mut i = 0;
    let mut j = 0;
    
    while i < left.len() && j < right.len() {
        if left[i] == right[j] {
            opcodes.push(Opcode::new(
                "equal",
                left_offset + i,
                left_offset + i + 1,
                right_offset + j,
                right_offset + j + 1,
            ));
            i += 1;
            j += 1;
        } else {
            // Try to find a match ahead
            let mut found_match = false;
            for di in 0..5.min(left.len() - i) {
                for dj in 0..5.min(right.len() - j) {
                    if left[i + di] == right[j + dj] {
                        // Found a match
                        if di > 0 && dj > 0 {
                            opcodes.push(Opcode::new(
                                "replace",
                                left_offset + i,
                                left_offset + i + di,
                                right_offset + j,
                                right_offset + j + dj,
                            ));
                        } else if di > 0 {
                            opcodes.push(Opcode::new(
                                "delete",
                                left_offset + i,
                                left_offset + i + di,
                                right_offset + j,
                                right_offset + j,
                            ));
                        } else if dj > 0 {
                            opcodes.push(Opcode::new(
                                "insert",
                                left_offset + i,
                                left_offset + i,
                                right_offset + j,
                                right_offset + j + dj,
                            ));
                        }
                        i += di;
                        j += dj;
                        found_match = true;
                        break;
                    }
                }
                if found_match {
                    break;
                }
            }
            
            if !found_match {
                opcodes.push(Opcode::new(
                    "replace",
                    left_offset + i,
                    left_offset + i + 1,
                    right_offset + j,
                    right_offset + j + 1,
                ));
                i += 1;
                j += 1;
            }
        }
    }
    
    if i < left.len() {
        opcodes.push(Opcode::new(
            "delete",
            left_offset + i,
            left_offset + left.len(),
            right_offset + j,
            right_offset + j,
        ));
    }
    if j < right.len() {
        opcodes.push(Opcode::new(
            "insert",
            left_offset + i,
            left_offset + i,
            right_offset + j,
            right_offset + right.len(),
        ));
    }
    
    opcodes
}

/// Merge adjacent opcodes of the same type
fn merge_adjacent_opcodes(opcodes: &[Opcode]) -> Vec<Opcode> {
    if opcodes.is_empty() {
        return Vec::new();
    }
    
    let mut merged = Vec::new();
    let mut current = opcodes[0].clone();
    
    for opcode in opcodes.iter().skip(1) {
        if opcode.tag == current.tag 
            && opcode.i1 == current.i2 
            && opcode.j1 == current.j2 
        {
            // Extend current opcode
            current.i2 = opcode.i2;
            current.j2 = opcode.j2;
        } else {
            merged.push(current);
            current = opcode.clone();
        }
    }
    merged.push(current);
    
    merged
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_patience_identical() {
        let algo = PatienceAlgorithm::new();
        let left = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let right = left.clone();
        let opcodes = algo.compute_diff(&left, &right);
        assert_eq!(opcodes.len(), 1);
        assert_eq!(opcodes[0].tag, "equal");
    }
    
    #[test]
    fn test_patience_simple_change() {
        let algo = PatienceAlgorithm::new();
        let left = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let right = vec!["a".to_string(), "x".to_string(), "c".to_string()];
        let opcodes = algo.compute_diff(&left, &right);
        assert!(opcodes.iter().any(|op| op.tag == "equal"));
    }
    
    #[test]
    fn test_unique_common_lines() {
        let left = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let right = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let unique = find_unique_common_lines(&left, &right);
        assert_eq!(unique.len(), 3);
    }
}
