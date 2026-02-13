// Diff Engine Module
// Provides pluggable diff algorithms and unified API

pub mod algorithm;
pub mod myers;
pub mod smart;
pub mod patience;
pub mod filters;
pub mod moved_blocks;
pub mod three_way;
pub mod api;

pub use api::{DiffEngine, AlgorithmType};
