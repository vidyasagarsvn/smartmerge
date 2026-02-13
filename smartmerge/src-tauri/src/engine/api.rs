use crate::models::{Opcode, DiffResult, ThreeWayDiffResult};
use super::algorithm::DiffAlgorithm;
use super::myers::MyersAlgorithm;
use super::smart::SmartAlgorithm;
use super::patience::PatienceAlgorithm;
use super::filters::{DiffOptions, apply_filters};
use super::moved_blocks::{MovedBlock, detect_moved_blocks};
use super::three_way::ThreeWayDiff;
use serde::{Serialize, Deserialize};

/// Available diff algorithm types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AlgorithmType {
    Myers,
    Smart,
    Patience,
}

impl Default for AlgorithmType {
    fn default() -> Self {
        Self::Myers
    }
}

/// Unified diff engine providing algorithm selection and execution
#[derive(Clone)]
pub struct DiffEngine {
    algorithm_type: AlgorithmType,
    options: DiffOptions,
}

impl DiffEngine {
    /// Create a new diff engine with the default algorithm (Myers)
    pub fn new() -> Self {
        Self {
            algorithm_type: AlgorithmType::default(),
            options: DiffOptions::default(),
        }
    }

    /// Create a new diff engine with a specific algorithm
    pub fn with_algorithm(algorithm_type: AlgorithmType) -> Self {
        Self { 
            algorithm_type,
            options: DiffOptions::default(),
        }
    }

    /// Create a new diff engine with algorithm and options
    pub fn with_options(algorithm_type: AlgorithmType, options: DiffOptions) -> Self {
        Self { algorithm_type, options }
    }

    /// Set the algorithm type
    pub fn set_algorithm(&mut self, algorithm_type: AlgorithmType) {
        self.algorithm_type = algorithm_type;
    }

    /// Get the current algorithm type
    pub fn algorithm_type(&self) -> AlgorithmType {
        self.algorithm_type
    }

    /// Set diff options
    pub fn set_options(&mut self, options: DiffOptions) {
        self.options = options;
    }

    /// Get current diff options
    pub fn options(&self) -> &DiffOptions {
        &self.options
    }

    /// Get the algorithm implementation
    fn get_algorithm(&self) -> Box<dyn DiffAlgorithm> {
        match self.algorithm_type {
            AlgorithmType::Myers => Box::new(MyersAlgorithm::new()),
            AlgorithmType::Smart => Box::new(SmartAlgorithm::new()),
            AlgorithmType::Patience => Box::new(PatienceAlgorithm::new()),
        }
    }

    /// Compute diff between two sequences of lines
    pub fn compute_diff(&self, left: &[String], right: &[String]) -> Vec<Opcode> {
        // Apply filters to input lines
        let (filtered_left, filtered_right) = apply_filters(left, right, &self.options);
        
        let algorithm = self.get_algorithm();
        let opcodes = algorithm.compute_diff(&filtered_left, &filtered_right);
        normalize_opcodes(&opcodes, left.len(), right.len())
    }

    /// Compute diff and return a complete DiffResult
    pub fn compute_diff_result(&self, left: Vec<String>, right: Vec<String>) -> DiffResult {
        let opcodes = self.compute_diff(&left, &right);
        let total_changes = opcodes.iter().filter(|op| op.tag != "equal").count();
        let (algorithm_name, _) = self.algorithm_info();
        
        DiffResult {
            left_lines: left,
            right_lines: right,
            opcodes,
            algorithm_used: Some(algorithm_name),
            total_changes: Some(total_changes),
            moved_blocks: None,
            options_used: Some((&self.options).into()),
        }
    }

    /// Detect moved blocks of code
    /// Returns a list of MovedBlock structures
    pub fn detect_moved_blocks(&self, left: &[String], right: &[String], min_block_size: usize) -> Vec<MovedBlock> {
        let opcodes = self.compute_diff(left, right);
        detect_moved_blocks(left, right, &opcodes, min_block_size)
    }

    /// Compute diff with moved block detection
    pub fn compute_diff_with_moves(&self, left: Vec<String>, right: Vec<String>, min_block_size: usize) -> (DiffResult, Vec<MovedBlock>) {
        let opcodes = self.compute_diff(&left, &right);
        let moved_blocks = detect_moved_blocks(&left, &right, &opcodes, min_block_size);
        let total_changes = opcodes.iter().filter(|op| op.tag != "equal").count();
        let (algorithm_name, _) = self.algorithm_info();
        
        let result = DiffResult {
            left_lines: left,
            right_lines: right,
            opcodes,
            algorithm_used: Some(algorithm_name),
            total_changes: Some(total_changes),
            moved_blocks: Some(moved_blocks.iter().map(|mb| mb.into()).collect()),
            options_used: Some((&self.options).into()),
        };
        
        (result, moved_blocks)
    }

    /// Perform three-way diff for merge operations
    /// Returns a ThreeWayDiffResult with conflict information
    pub fn three_way_diff(
        &self,
        base: Vec<String>,
        left: Vec<String>,
        right: Vec<String>,
    ) -> ThreeWayDiffResult {
        let three_way = ThreeWayDiff::new(self.clone());
        three_way.diff(&base, &left, &right)
    }

    /// Get information about the current algorithm
    pub fn algorithm_info(&self) -> (String, String) {
        let algorithm = self.get_algorithm();
        (algorithm.name().to_string(), algorithm.description().to_string())
    }

    /// Get list of all available algorithms
    pub fn available_algorithms() -> Vec<(AlgorithmType, &'static str, &'static str)> {
        vec![
            (AlgorithmType::Myers, "Myers", "Myers diff algorithm - optimal edit distance with linear space"),
            (AlgorithmType::Smart, "Smart", "Smart greedy diff algorithm - fast with good heuristics for similar files"),
            (AlgorithmType::Patience, "Patience", "Patience diff algorithm - produces more intuitive diffs using unique line matching"),
        ]
    }
}

impl Default for DiffEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Normalize opcodes to ensure bounds are valid
fn normalize_opcodes(opcodes: &[Opcode], left_len: usize, right_len: usize) -> Vec<Opcode> {
    let mut normalized = Vec::new();
    for opcode in opcodes {
        let i1 = opcode.i1.min(left_len);
        let i2 = opcode.i2.min(left_len);
        let j1 = opcode.j1.min(right_len);
        let j2 = opcode.j2.min(right_len);
        if i2 < i1 || j2 < j1 {
            continue;
        }
        if opcode.tag == "equal" && i2 == i1 && j2 == j1 {
            continue;
        }
        normalized.push(Opcode::new(&opcode.tag, i1, i2, j1, j2));
    }
    normalized
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_engine_default() {
        let engine = DiffEngine::new();
        assert_eq!(engine.algorithm_type(), AlgorithmType::Myers);
    }

    #[test]
    fn test_diff_engine_with_algorithm() {
        let engine = DiffEngine::with_algorithm(AlgorithmType::Smart);
        assert_eq!(engine.algorithm_type(), AlgorithmType::Smart);
    }

    #[test]
    fn test_compute_diff_identical() {
        let engine = DiffEngine::new();
        let left = vec!["line1".to_string(), "line2".to_string()];
        let right = vec!["line1".to_string(), "line2".to_string()];
        let opcodes = engine.compute_diff(&left, &right);
        assert_eq!(opcodes.len(), 1);
        assert_eq!(opcodes[0].tag, "equal");
    }

    #[test]
    fn test_compute_diff_different() {
        let engine = DiffEngine::new();
        let left = vec!["line1".to_string()];
        let right = vec!["line2".to_string()];
        let opcodes = engine.compute_diff(&left, &right);
        assert!(!opcodes.is_empty());
    }

    #[test]
    fn test_available_algorithms() {
        let algorithms = DiffEngine::available_algorithms();
        assert_eq!(algorithms.len(), 3);
    }
}
