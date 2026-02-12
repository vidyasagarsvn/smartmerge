use crate::models::Opcode;

pub fn smart_diff(left: &[String], right: &[String]) -> Vec<Opcode> {
    let mut opcodes = Vec::new();
    let mut i = 0usize;
    let mut j = 0usize;

    while i < left.len() && j < right.len() {
        if left[i] == right[j] {
            let match_start_i = i;
            let match_start_j = j;
            while i < left.len() && j < right.len() && left[i] == right[j] {
                i += 1;
                j += 1;
            }
            if match_start_i < i {
                opcodes.push(Opcode::new("equal", match_start_i, i, match_start_j, j));
            }
        } else {
            let left_match_idx = find_next_match(left, &right[j], i);
            let right_match_idx = find_next_match(right, &left[i], j);

            if left_match_idx.is_none() && right_match_idx.is_none() {
                opcodes.push(Opcode::new("replace", i, i + 1, j, j + 1));
                i += 1;
                j += 1;
            } else if left_match_idx.is_some()
                && (right_match_idx.is_none()
                    || left_match_idx.unwrap() - i <= right_match_idx.unwrap() - j)
            {
                let next_i = left_match_idx.unwrap();
                opcodes.push(Opcode::new("delete", i, next_i, j, j));
                i = next_i;
            } else {
                let next_j = right_match_idx.unwrap();
                opcodes.push(Opcode::new("insert", i, i, j, next_j));
                j = next_j;
            }
        }
    }

    if i < left.len() {
        opcodes.push(Opcode::new("delete", i, left.len(), j, j));
    }
    if j < right.len() {
        opcodes.push(Opcode::new("insert", i, i, j, right.len()));
    }

    opcodes
}

fn find_next_match(lines: &[String], target: &str, start: usize) -> Option<usize> {
    for (idx, line) in lines.iter().enumerate().skip(start) {
        if line == target {
            return Some(idx);
        }
    }
    None
}
