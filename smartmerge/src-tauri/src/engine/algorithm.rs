use crate::models::Opcode;

/// Trait for pluggable diff algorithms
pub trait DiffAlgorithm {
    /// Compute diff between two sequences of lines
    /// Returns a list of opcodes describing the differences
    fn compute_diff(&self, left: &[String], right: &[String]) -> Vec<Opcode>;
    
    /// Get the name of the algorithm
    fn name(&self) -> &str;
    
    /// Get a description of the algorithm
    fn description(&self) -> &str;
}
