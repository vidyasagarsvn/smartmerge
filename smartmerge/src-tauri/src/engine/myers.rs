use std::collections::HashMap;
use crate::models::Opcode;
use super::algorithm::DiffAlgorithm;

/// Myers diff algorithm implementation
/// Based on "An O(ND) Difference Algorithm and Its Variations" by Eugene W. Myers
pub struct MyersAlgorithm;

impl MyersAlgorithm {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MyersAlgorithm {
    fn default() -> Self {
        Self::new()
    }
}

impl DiffAlgorithm for MyersAlgorithm {
    fn compute_diff(&self, left: &[String], right: &[String]) -> Vec<Opcode> {
        myers_opcodes(left, right)
    }
    
    fn name(&self) -> &str {
        "Myers"
    }
    
    fn description(&self) -> &str {
        "Myers diff algorithm - optimal edit distance with linear space"
    }
}

fn myers_opcodes(left: &[String], right: &[String]) -> Vec<Opcode> {
    let n = left.len() as i32;
    let m = right.len() as i32;
    let maxd = n + m;
    let mut v: HashMap<i32, i32> = HashMap::new();
    let mut trace: Vec<HashMap<i32, i32>> = Vec::new();

    v.insert(0, 0);

    for d in 0..=maxd {
        let mut v_next: HashMap<i32, i32> = HashMap::new();
        let mut k = -d;
        while k <= d {
            let x = if k == -d
                || (k != d && v.get(&(k - 1)).unwrap_or(&-1) < v.get(&(k + 1)).unwrap_or(&-1))
            {
                *v.get(&(k + 1)).unwrap_or(&0)
            } else {
                v.get(&(k - 1)).unwrap_or(&0) + 1
            };
            let mut x_mut = x;
            let mut y_mut = x_mut - k;
            while x_mut < n && y_mut < m {
                let li = x_mut as usize;
                let ri = y_mut as usize;
                if left[li] != right[ri] {
                    break;
                }
                x_mut += 1;
                y_mut += 1;
            }
            v_next.insert(k, x_mut);
            if x_mut >= n && y_mut >= m {
                trace.push(v_next);
                return build_opcodes_from_trace(&trace, n, m);
            }
            k += 2;
        }
        trace.push(v_next.clone());
        v = v_next;
    }

    vec![Opcode::new("equal", 0, n as usize, 0, m as usize)]
}

fn build_opcodes_from_trace(trace: &[HashMap<i32, i32>], n: i32, m: i32) -> Vec<Opcode> {
    let mut x = n;
    let mut y = m;
    let mut edits: Vec<Opcode> = Vec::new();

    for d in (0..trace.len()).rev() {
        let v = &trace[d];
        let v_prev = if d > 0 {
            &trace[d - 1]
        } else {
            static EMPTY: once_cell::sync::Lazy<HashMap<i32, i32>> =
                once_cell::sync::Lazy::new(HashMap::new);
            &EMPTY
        };
        let k = x - y;
        let (k_prev, op) = if k == -(d as i32)
            || (k != d as i32
                && v.get(&(k - 1)).unwrap_or(&-1) < v.get(&(k + 1)).unwrap_or(&-1))
        {
            (k + 1, "insert")
        } else {
            (k - 1, "delete")
        };

        let x_prev = *v_prev.get(&k_prev).unwrap_or(&0);
        let y_prev = x_prev - k_prev;

        while x > x_prev && y > y_prev {
            edits.push(Opcode::new(
                "equal",
                (x - 1) as usize,
                x as usize,
                (y - 1) as usize,
                y as usize,
            ));
            x -= 1;
            y -= 1;
        }

        if d == 0 {
            break;
        }

        if op == "delete" {
            edits.push(Opcode::new(
                "delete",
                x_prev as usize,
                (x_prev + 1) as usize,
                y_prev as usize,
                y_prev as usize,
            ));
        } else {
            edits.push(Opcode::new(
                "insert",
                x_prev as usize,
                x_prev as usize,
                y_prev as usize,
                (y_prev + 1) as usize,
            ));
        }

        x = x_prev;
        y = y_prev;
    }

    edits.reverse();
    merge_opcodes(&edits)
}

fn merge_opcodes(edits: &[Opcode]) -> Vec<Opcode> {
    if edits.is_empty() {
        return Vec::new();
    }

    let mut merged: Vec<Opcode> = Vec::new();
    let mut current = edits[0].clone();

    for edit in edits.iter().skip(1) {
        if edit.tag == current.tag && edit.i1 == current.i2 && edit.j1 == current.j2 {
            current.i2 = edit.i2;
            current.j2 = edit.j2;
        } else {
            merged.push(current);
            current = edit.clone();
        }
    }
    merged.push(current);

    let mut normalized: Vec<Opcode> = Vec::new();
    let mut idx = 0;
    while idx < merged.len() {
        let cur = &merged[idx];
        if cur.tag == "delete" && idx + 1 < merged.len() {
            let next = &merged[idx + 1];
            if next.tag == "insert" && cur.i2 == next.i1 && cur.j1 == next.j1 {
                normalized.push(Opcode::new("replace", cur.i1, cur.i2, cur.j1, next.j2));
                idx += 2;
                continue;
            }
        }
        normalized.push(cur.clone());
        idx += 1;
    }

    normalized
}
