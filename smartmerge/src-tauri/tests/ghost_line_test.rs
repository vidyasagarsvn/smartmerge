use smartmerge_lib::line_mapping::{compute_line_mappings, GhostLineLayout};
use smartmerge_lib::models::Opcode;

#[test]
fn test_ghost_lines_with_alignment_example() {
    // Simple test case: left has 3 lines in a block, right has 1 line
    // Should create 2 ghost lines on right side to align them
    
    let left_lines = vec![
        "line 1".to_string(),
        "line 2".to_string(),
        "line 3".to_string(),
    ];
    
    let right_lines = vec![
        "line A".to_string(),
    ];
    
    // Single replace opcode: left[0:3] -> right[0:1]
    let opcodes = vec![
        Opcode {
            tag: "replace".to_string(),
            i1: 0,
            i2: 3,
            j1: 0,
            j2: 1,
            is_trivial: false,
            moved_from: None,
            moved_to: None,
        }
    ];
    
    let layouts = compute_line_mappings(&left_lines, &right_lines, &opcodes);
    
    assert_eq!(layouts.len(), 1, "Should have 1 layout for 1 opcode");
    
    let layout = &layouts[0];
    
    // Left should have 3 real mappings
    assert_eq!(layout.left_mappings.len(), 3, "Left should have 3 lines");
    assert!(layout.left_mappings.iter().all(|m| m.maps_to.is_some()), 
            "Not all left lines were mapped");
    
    // Right should have 3 total (1 real + 2 ghosts) to match left's 3 lines
    assert_eq!(layout.right_mappings.len(), 3, "Right should have 3 entries (1 real + 2 ghosts)");
    
    // Count ghosts on right side
    let ghost_count = layout.right_mappings.iter().filter(|m| m.maps_to.is_none()).count();
    assert_eq!(ghost_count, 2, "Right should have 2 ghost lines");
    
    // One real line on right
    let real_count = layout.right_mappings.iter().filter(|m| m.maps_to.is_some()).count();
    assert_eq!(real_count, 1, "Right should have 1 real line");
}

#[test]
fn test_ghost_lines_complex_alignment() {
    // More complex: left has different content at different positions
    let left_lines = vec![
        "func header()".to_string(),
        "    return 'left'".to_string(),
        "".to_string(),
        "def left_only():".to_string(),
        "    return 'left'".to_string(),
        "".to_string(),
        "class Widget:".to_string(),
        "    def render(self):".to_string(),
        "        pass".to_string(),
    ];
    
    let right_lines = vec![
        "func header()".to_string(),
        "    return 'right'".to_string(),
        "".to_string(),
        "def right_only():".to_string(),
        "    return 'right'".to_string(),
        "".to_string(),
        "class Widget:".to_string(),
        "    def render(self):".to_string(),
        "        pass".to_string(),
    ];
    
    // One big replace opcode for the middle section
    let opcodes = vec![
        Opcode {
            tag: "equal".to_string(),
            i1: 0,
            i2: 3,
            j1: 0,
            j2: 3,
            is_trivial: false,
            moved_from: None,
            moved_to: None,
        },
        Opcode {
            tag: "replace".to_string(),
            i1: 3,
            i2: 6,
            j1: 3,
            j2: 6,
            is_trivial: false,
            moved_from: None,
            moved_to: None,
        },
        Opcode {
            tag: "equal".to_string(),
            i1: 6,
            i2: 9,
            j1: 6,
            j2: 9,
            is_trivial: false,
            moved_from: None,
            moved_to: None,
        },
    ];
    
    let layouts = compute_line_mappings(&left_lines, &right_lines, &opcodes);
    
    assert_eq!(layouts.len(), 3, "Should have 3 layouts for 3 opcodes");
    
    // First equal block - no ghosts needed
    assert_eq!(layouts[0].left_mappings.len(), 3);
    assert_eq!(layouts[0].right_mappings.len(), 3);
    
    // Middle replace block - might have some alignment
    let middle = &layouts[1];
    assert_eq!(middle.left_mappings.len(), middle.right_mappings.len(), 
               "WinMerge should align both sides to same display height");
    
    // Last equal block - no ghosts needed
    assert_eq!(layouts[2].left_mappings.len(), 3);
    assert_eq!(layouts[2].right_mappings.len(), 3);
}
