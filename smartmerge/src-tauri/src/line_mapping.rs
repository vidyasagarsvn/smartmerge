/// WinMerge-style intelligent line mapping with ghost lines
/// 
/// This module implements the algorithm from WinMerge's MergeDocDiffSync.cpp
/// to create optimal line-to-line mappings within diff blocks using Levenshtein distance.

use serde::{Deserialize, Serialize};
use crate::models::Opcode;

/// Line mapping result for a single diff block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineMapping {
    /// Index in the diff block (0-based, relative to block start)
    pub index: usize,
    /// Maps to line index in other side, or None for ghost line
    pub maps_to: Option<usize>,
}

/// Complete ghost line layout for both sides
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhostLineLayout {
    /// Mappings for left side (index → right side index or ghost)
    pub left_mappings: Vec<LineMapping>,
    /// Mappings for right side (index → left side index or ghost)
    pub right_mappings: Vec<LineMapping>,
}

/// Compute Levenshtein (edit) distance between two strings
/// This is the cost of making them equal (similarity measure)
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.chars().count();
    let len2 = s2.chars().count();
    
    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }
    
    let mut prev_row: Vec<usize> = (0..=len2).collect();
    let mut curr_row = vec![0; len2 + 1];
    
    let chars1: Vec<char> = s1.chars().collect();
    let chars2: Vec<char> = s2.chars().collect();
    
    for (i, c1) in chars1.iter().enumerate() {
        curr_row[0] = i + 1;
        
        for (j, c2) in chars2.iter().enumerate() {
            let cost = if c1 == c2 { 0 } else { 1 };
            curr_row[j + 1] = (curr_row[j] + 1)           // insertion
                .min(prev_row[j + 1] + 1)                 // deletion
                .min(prev_row[j] + cost);                 // substitution
        }
        
        std::mem::swap(&mut prev_row, &mut curr_row);
    }
    
    prev_row[len2]
}

/// Get matching cost between two lines (lower is better match)
/// Returns negative of similarity to use as cost (WinMerge approach)
fn get_match_cost(line1: &str, line2: &str) -> i32 {
    let dist = levenshtein_distance(line1, line2);
    // Return negative of (max_len - distance) so better matches have lower cost
    let max_len = line1.len().max(line2.len());
    -(max_len as i32 - dist as i32)
}

/// Recursively map lines within a diff block to find best matches
/// This is the core WinMerge algorithm from AdjustDiffBlock
fn adjust_diff_block(
    left_lines: &[String],
    right_lines: &[String],
    lo0: usize,
    hi0: usize,
    lo1: usize,
    hi1: usize,
    map: &mut Vec<Option<usize>>,
    map_start: usize,
) {
    let lines0 = hi0 - lo0 + 1;
    let lines1 = hi1 - lo1 + 1;
    
    // Base case: 1-to-1 mapping
    if lines0 == 1 && lines1 == 1 {
            let rel = lo0 - map_start;
        if rel < map.len() {
            map[rel] = Some(lo1);
        }
        return;
    }
    
    // Bail out if range is large (WinMerge uses 15 line limit)
    // Do simple sequential mapping
    if lines0 > 15 || lines1 > 15 {
        for w in 0..lines0 {
                let rel = lo0 + w - map_start;
            if rel < map.len() {
                if w < lines1 {
                    map[rel] = Some(lo1 + w);
                } else {
                    map[rel] = None; // Ghost line
                }
            }
        }
        return;
    }
    
    // Find best matching line pair using Levenshtein distance
    let mut best_i = lo0;
    let mut best_j = lo1;
    let mut best_cost = i32::MAX;
    
    for i in lo0..=hi0 {
        for j in lo1..=hi1 {
            let cost = get_match_cost(&left_lines[i], &right_lines[j]);
            if cost < best_cost {
                best_i = i;
                best_j = j;
                best_cost = cost;
            }
        }
    }
    
    // Map the best match
        let rel = best_i - map_start;
    if rel < map.len() {
        map[rel] = Some(best_j);
    }
    
    // Recursively solve the problem above and below the match
    
    // Problem below the match
    if lo0 < best_i {
        if lo1 < best_j {
            adjust_diff_block(left_lines, right_lines, lo0, best_i - 1, lo1, best_j - 1, map, map_start);
        } else {
            // No target space for lines below match - mark as ghost
            for x in lo0..best_i {
                    let rel = x - map_start;
                if rel < map.len() {
                    map[rel] = None;
                }
            }
        }
    }
    
    // Problem above the match
    if best_i < hi0 {
        if best_j < hi1 {
            adjust_diff_block(left_lines, right_lines, best_i + 1, hi0, best_j + 1, hi1, map, map_start);
        } else {
            // No target space for lines above match - mark as ghost
            for x in (best_i + 1)..=hi0 {
                    let rel = x - map_start;
                if rel < map.len() {
                    map[rel] = None;
                }
            }
        }
    }
}

/// Compute intelligent line mappings for all diff blocks
/// This creates ghost line placements that align similar content
pub fn compute_line_mappings(
    left_lines: &[String],
    right_lines: &[String],
    opcodes: &[Opcode],
) -> Vec<GhostLineLayout> {
    let mut layouts = Vec::new();
    
    for opcode in opcodes {
        let left_count = opcode.i2 - opcode.i1;
        let right_count = opcode.j2 - opcode.j1;
        
        // For equal blocks, create 1:1 mappings up to min count, then ghost lines for extras
        if opcode.tag == "equal" {
            let mut left_mappings = Vec::new();
            let mut right_mappings = Vec::new();
            let min_count = left_count.min(right_count);
            // Map up to min count
            for idx in 0..min_count {
                left_mappings.push(LineMapping {
                    index: opcode.i1 + idx,
                    maps_to: Some(opcode.j1 + idx),
                });
                right_mappings.push(LineMapping {
                    index: opcode.j1 + idx,
                    maps_to: Some(opcode.i1 + idx),
                });
            }
            // Extra left lines become ghosts on right
            for idx in min_count..left_count {
                left_mappings.push(LineMapping {
                    index: opcode.i1 + idx,
                    maps_to: None,
                });
            }
            // Extra right lines become ghosts on left
            for idx in min_count..right_count {
                right_mappings.push(LineMapping {
                    index: opcode.j1 + idx,
                    maps_to: None,
                });
            }
            layouts.push(GhostLineLayout {
                left_mappings,
                right_mappings,
            });
            continue;
        }
        
        // Skip if either side is empty (pure insert/delete)
        if left_count == 0 || right_count == 0 {
            // For pure insert (no left lines), create ghost mappings
            if left_count == 0 && right_count > 0 {
                let mut left_mappings = Vec::new();
                let mut right_mappings = Vec::new();
                
                // Create ghosts on left side for all right lines
                for idx in 0..right_count {
                    left_mappings.push(LineMapping {
                        index: opcode.i1,  // Dummy index
                        maps_to: None,  // Ghost line
                    });
                    right_mappings.push(LineMapping {
                        index: opcode.j1 + idx,
                        maps_to: None,  // No corresponding left line
                    });
                }
                
                layouts.push(GhostLineLayout {
                    left_mappings,
                    right_mappings,
                });
            }
            // For pure delete (no right lines), create ghost mappings
            else if right_count == 0 && left_count > 0 {
                let mut left_mappings = Vec::new();
                let mut right_mappings = Vec::new();
                
                // Create ghosts on right side for all left lines
                for idx in 0..left_count {
                    left_mappings.push(LineMapping {
                        index: opcode.i1 + idx,
                        maps_to: None,  // No corresponding right line
                    });
                    right_mappings.push(LineMapping {
                        index: opcode.j1,  // Dummy index
                        maps_to: None,  // Ghost line
                    });
                }
                
                layouts.push(GhostLineLayout {
                    left_mappings,
                    right_mappings,
                });
            }
            continue;
        }
        
        // Initialize mapping (None = unmapped/ghost)
        let mut left_map: Vec<Option<usize>> = vec![None; left_count];
        
        // Build the mapping using recursive algorithm
        adjust_diff_block(
            left_lines,
            right_lines,
            opcode.i1,
            opcode.i2 - 1,
            opcode.j1,
            opcode.j2 - 1,
            &mut left_map,
                opcode.i1,
            );
        
        // Convert to LineMapping structures
        let mut left_mappings = Vec::new();
        let mut right_mappings = Vec::new();
        
        for (idx, &mapped_to) in left_map.iter().enumerate() {
            left_mappings.push(LineMapping {
                index: opcode.i1 + idx,
                maps_to: mapped_to,
            });
        }
        
        // Build reverse mapping for right side
        let mut right_map: Vec<Option<usize>> = vec![None; right_count];
        for (left_idx, &right_idx_opt) in left_map.iter().enumerate() {
            if let Some(right_idx) = right_idx_opt {
                let relative_right_idx = right_idx - opcode.j1;
                if relative_right_idx < right_count {
                    right_map[relative_right_idx] = Some(opcode.i1 + left_idx);
                }
            }
        }
        
        for (idx, &mapped_to) in right_map.iter().enumerate() {
            right_mappings.push(LineMapping {
                index: opcode.j1 + idx,
                maps_to: mapped_to,
            });
        }
        
        layouts.push(GhostLineLayout {
            left_mappings,
            right_mappings,
        });
    }
    
    layouts
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("", ""), 0);
        assert_eq!(levenshtein_distance("abc", "abc"), 0);
        assert_eq!(levenshtein_distance("abc", "abd"), 1);
        assert_eq!(levenshtein_distance("abc", ""), 3);
        assert_eq!(levenshtein_distance("", "xyz"), 3);
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
    }
    
    #[test]
    fn test_get_match_cost() {
        // Identical lines have best (most negative) cost
        let cost1 = get_match_cost("foo bar", "foo bar");
        let cost2 = get_match_cost("foo bar", "foo baz");
        assert!(cost1 < cost2); // Better match has lower cost
    }
}
