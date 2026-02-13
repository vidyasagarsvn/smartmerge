/// Trivial Diff Detection and Filtering
///
/// This module provides functionality to detect and filter trivial changes
/// such as whitespace-only modifications, case-only changes, and line ending differences.

use crate::models::{DiffResult, Opcode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrivialChangeStats {
    /// Total number of change blocks (non-equal opcodes)
    pub total_changes: usize,
    /// Number of trivial changes detected
    pub trivial_count: usize,
    /// Number of non-trivial changes
    pub non_trivial_count: usize,
    /// Percentage of changes that are trivial (0-100)
    pub trivial_percentage: f32,
    /// Breakdown by change type
    pub breakdown: ChangeTypeBreakdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeTypeBreakdown {
    /// Whitespace-only changes
    pub whitespace_only: usize,
    /// Case-only changes
    pub case_only: usize,
    /// Line ending changes (CRLF vs LF)
    pub line_endings: usize,
    /// Tab vs space changes
    pub tab_space: usize,
    /// Other trivial changes
    pub other_trivial: usize,
}

impl Default for ChangeTypeBreakdown {
    fn default() -> Self {
        Self {
            whitespace_only: 0,
            case_only: 0,
            line_endings: 0,
            tab_space: 0,
            other_trivial: 0,
        }
    }
}

/// Analyze and mark trivial changes in a DiffResult
pub fn analyze_trivial_changes(
    result: &mut DiffResult,
    treat_case_as_trivial: bool,
) -> TrivialChangeStats {
    let mut trivial_count = 0;
    let mut breakdown = ChangeTypeBreakdown::default();

    for opcode in &mut result.opcodes {
        if opcode.tag == "equal" {
            continue;
        }

        if let Some(is_trivial) = is_change_trivial(
            opcode,
            &result.left_lines,
            &result.right_lines,
            treat_case_as_trivial,
        ) {
            opcode.is_trivial = Some(is_trivial);
            if is_trivial {
                trivial_count += 1;
                categorize_trivial_change(opcode, &result.left_lines, &result.right_lines, &mut breakdown);
            }
        }
    }

    let total_changes = result.opcodes.iter().filter(|op| op.tag != "equal").count();
    let non_trivial_count = total_changes.saturating_sub(trivial_count);
    let trivial_percentage = if total_changes > 0 {
        (trivial_count as f32 / total_changes as f32) * 100.0
    } else {
        0.0
    };

    TrivialChangeStats {
        total_changes,
        trivial_count,
        non_trivial_count,
        trivial_percentage,
        breakdown,
    }
}

/// Check if a specific change is trivial
fn is_change_trivial(
    opcode: &Opcode,
    left_lines: &[String],
    right_lines: &[String],
    treat_case_as_trivial: bool,
) -> Option<bool> {
    match opcode.tag.as_str() {
        "delete" | "insert" => {
            // Single line insertions/deletions are rarely trivial
            let line_count = opcode.i2.saturating_sub(opcode.i1).max(opcode.j2.saturating_sub(opcode.j1));
            if line_count > 1 {
                return Some(false);
            }
            
            // Check if it's a single whitespace line
            let lines = if opcode.tag == "delete" {
                &left_lines[opcode.i1..opcode.i2]
            } else {
                &right_lines[opcode.j1..opcode.j2]
            };
            
            if lines.is_empty() {
                return Some(false);
            }
            
            let is_whitespace_only = lines.iter().all(|line| line.trim().is_empty());
            Some(is_whitespace_only)
        }
        "replace" => {
            // Compare left and right lines to determine if trivial
            if opcode.i2 - opcode.i1 != opcode.j2 - opcode.j1 {
                return Some(false); // Different number of lines -> not trivial
            }

            let left_block = &left_lines[opcode.i1..opcode.i2];
            let right_block = &right_lines[opcode.j1..opcode.j2];

            // Check each pair of lines
            let all_trivial = left_block.iter().zip(right_block).all(|(l, r)| {
                is_line_change_trivial(l, r, treat_case_as_trivial)
            });

            Some(all_trivial)
        }
        _ => None,
    }
}

/// Check if a single line change is trivial
fn is_line_change_trivial(left: &str, right: &str, treat_case_as_trivial: bool) -> bool {
    if left == right {
        return true;
    }

    // Check for whitespace-only changes
    if left.trim() == right.trim() {
        return true;
    }

    // Check for case-only changes
    if treat_case_as_trivial && left.to_lowercase() == right.to_lowercase() {
        return true;
    }

    // Check for line ending changes (CR/LF)
    let left_normalized = left.replace("\r\n", "\n").replace('\r', "\n");
    let right_normalized = right.replace("\r\n", "\n").replace('\r', "\n");
    if left_normalized == right_normalized {
        return true;
    }

    // Check for tab vs space changes (both represent indentation)
    let left_expanded = left.replace('\t', "  ");
    let right_expanded = right.replace('\t', "  ");
    if left_expanded == right_expanded && left_expanded.trim() == right_expanded.trim() {
        return true;
    }

    false
}

/// Categorize trivial change types for statistics
fn categorize_trivial_change(
    opcode: &Opcode,
    left_lines: &[String],
    right_lines: &[String],
    breakdown: &mut ChangeTypeBreakdown,
) {
    match opcode.tag.as_str() {
        "delete" | "insert" => {
            let lines = if opcode.tag == "delete" {
                &left_lines[opcode.i1..opcode.i2]
            } else {
                &right_lines[opcode.j1..opcode.j2]
            };

            if let Some(line) = lines.first() {
                if line.trim().is_empty() {
                    breakdown.whitespace_only += 1;
                }
            }
        }
        "replace" => {
            let left_block = &left_lines[opcode.i1..opcode.i2];
            let right_block = &right_lines[opcode.j1..opcode.j2];

            if let Some((l, r)) = left_block.first().zip(right_block.first()) {
                if categorize_line_change(l, r) == "case_only" {
                    breakdown.case_only += 1;
                } else if categorize_line_change(l, r) == "whitespace_only" {
                    breakdown.whitespace_only += 1;
                } else if categorize_line_change(l, r) == "line_endings" {
                    breakdown.line_endings += 1;
                } else if categorize_line_change(l, r) == "tab_space" {
                    breakdown.tab_space += 1;
                } else {
                    breakdown.other_trivial += 1;
                }
            }
        }
        _ => {}
    }
}

/// Categorize the type of line change
fn categorize_line_change(left: &str, right: &str) -> &'static str {
    if left.to_lowercase() == right.to_lowercase() {
        return "case_only";
    }

    let left_normalized = left.replace("\r\n", "\n").replace('\r', "\n");
    let right_normalized = right.replace("\r\n", "\n").replace('\r', "\n");
    if left_normalized == right_normalized {
        return "line_endings";
    }

    if left.trim() == right.trim() {
        return "whitespace_only";
    }

    let left_expanded = left.replace('\t', "  ");
    let right_expanded = right.replace('\t', "  ");
    if left_expanded == right_expanded {
        return "tab_space";
    }

    "other_trivial"
}

/// Filter opcodes to hide trivial changes
pub fn filter_trivial_opcodes(opcodes: &[Opcode]) -> Vec<Opcode> {
    opcodes
        .iter()
        .filter(|op| {
            // Keep equal blocks and non-trivial changes
            op.tag == "equal" || op.is_trivial != Some(true)
        })
        .cloned()
        .collect()
}

/// Get only trivial opcodes
pub fn get_trivial_opcodes(opcodes: &[Opcode]) -> Vec<Opcode> {
    opcodes
        .iter()
        .filter(|op| op.is_trivial == Some(true))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whitespace_only_trivial() {
        assert!(is_line_change_trivial("  code  ", "code", false));
        assert!(is_line_change_trivial("code", "code  ", false));
    }

    #[test]
    fn test_case_change_trivial() {
        assert!(is_line_change_trivial("Code", "code", true));
        assert!(!is_line_change_trivial("Code", "code", false));
    }

    #[test]
    fn test_line_ending_trivial() {
        assert!(is_line_change_trivial("line\r\n", "line\n", false));
    }

    #[test]
    fn test_tab_space_trivial() {
        assert!(is_line_change_trivial("\tcode", "  code", false));
    }
}
