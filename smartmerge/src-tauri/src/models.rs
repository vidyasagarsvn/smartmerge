use serde::{Deserialize, Serialize};

/// Type of diff block for classification
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BlockType {
    /// Lines that are equal
    Equal,
    /// Lines that were inserted
    Insert,
    /// Lines that were deleted
    Delete,
    /// Lines that were replaced/changed
    Replace,
    /// Lines that were moved (detected by moved block analysis)
    Moved,
}

impl BlockType {
    pub fn from_tag(tag: &str) -> Self {
        match tag {
            "equal" => BlockType::Equal,
            "insert" => BlockType::Insert,
            "delete" => BlockType::Delete,
            "replace" => BlockType::Replace,
            "moved" => BlockType::Moved,
            _ => BlockType::Replace, // fallback
        }
    }

    pub fn to_tag(&self) -> &str {
        match self {
            BlockType::Equal => "equal",
            BlockType::Insert => "insert",
            BlockType::Delete => "delete",
            BlockType::Replace => "replace",
            BlockType::Moved => "moved",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Opcode {
    pub tag: String,
    pub i1: usize,
    pub i2: usize,
    pub j1: usize,
    pub j2: usize,
    
    // Enhanced metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_type: Option<BlockType>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_trivial: Option<bool>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_before: Option<usize>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_after: Option<usize>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moved_from: Option<(usize, usize)>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moved_to: Option<(usize, usize)>,
}

impl Opcode {
    pub fn new(tag: &str, i1: usize, i2: usize, j1: usize, j2: usize) -> Self {
        Self {
            tag: tag.to_string(),
            i1,
            i2,
            j1,
            j2,
            block_type: Some(BlockType::from_tag(tag)),
            is_trivial: None,
            context_before: None,
            context_after: None,
            moved_from: None,
            moved_to: None,
        }
    }

    pub fn with_metadata(
        tag: &str,
        i1: usize,
        i2: usize,
        j1: usize,
        j2: usize,
        is_trivial: bool,
    ) -> Self {
        Self {
            tag: tag.to_string(),
            i1,
            i2,
            j1,
            j2,
            block_type: Some(BlockType::from_tag(tag)),
            is_trivial: Some(is_trivial),
            context_before: None,
            context_after: None,
            moved_from: None,
            moved_to: None,
        }
    }

    pub fn with_moved(
        tag: &str,
        i1: usize,
        i2: usize,
        j1: usize,
        j2: usize,
        moved_from: Option<(usize, usize)>,
        moved_to: Option<(usize, usize)>,
    ) -> Self {
        Self {
            tag: tag.to_string(),
            i1,
            i2,
            j1,
            j2,
            block_type: Some(BlockType::Moved),
            is_trivial: None,
            context_before: None,
            context_after: None,
            moved_from,
            moved_to,
        }
    }

    pub fn mark_as_trivial(&mut self) {
        self.is_trivial = Some(true);
    }

    pub fn mark_as_moved(&mut self, from: Option<(usize, usize)>, to: Option<(usize, usize)>) {
        self.block_type = Some(BlockType::Moved);
        self.moved_from = from;
        self.moved_to = to;
    }

    pub fn set_context(&mut self, before: usize, after: usize) {
        self.context_before = Some(before);
        self.context_after = Some(after);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResult {
    pub left_lines: Vec<String>,
    pub right_lines: Vec<String>,
    pub opcodes: Vec<Opcode>,
    
    // Enhanced metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub algorithm_used: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_changes: Option<usize>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moved_blocks: Option<Vec<MovedBlockInfo>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options_used: Option<DiffOptionsInfo>,
}

impl DiffResult {
    pub fn new(left_lines: Vec<String>, right_lines: Vec<String>, opcodes: Vec<Opcode>) -> Self {
        let total_changes = opcodes.iter().filter(|op| op.tag != "equal").count();
        
        Self {
            left_lines,
            right_lines,
            opcodes,
            algorithm_used: None,
            total_changes: Some(total_changes),
            moved_blocks: None,
            options_used: None,
        }
    }

    pub fn with_metadata(
        left_lines: Vec<String>,
        right_lines: Vec<String>,
        opcodes: Vec<Opcode>,
        algorithm: &str,
    ) -> Self {
        let total_changes = opcodes.iter().filter(|op| op.tag != "equal").count();
        
        Self {
            left_lines,
            right_lines,
            opcodes,
            algorithm_used: Some(algorithm.to_string()),
            total_changes: Some(total_changes),
            moved_blocks: None,
            options_used: None,
        }
    }
}

/// Information about moved blocks for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovedBlockInfo {
    pub left_start: usize,
    pub left_end: usize,
    pub right_start: usize,
    pub right_end: usize,
    pub size: usize,
}

/// Diff options info for result metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffOptionsInfo {
    pub ignore_case: bool,
    pub ignore_whitespace: String,
    pub ignore_blank_lines: bool,
    pub ignore_comments: bool,
}

// Three-way diff structures

/// Conflict type in three-way merge
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ConflictType {
    /// No conflict, clean merge
    None,
    /// Both sides changed differently
    BothModified,
    /// One side deleted, other modified
    DeleteModify,
    /// One side modified, other deleted
    ModifyDelete,
    /// Both sides added different content
    BothAdded,
}

/// Three-way diff opcode for merge operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreeWayOpcode {
    /// Base (ancestor) range
    pub base_start: usize,
    pub base_end: usize,
    
    /// Left (ours) range
    pub left_start: usize,
    pub left_end: usize,
    
    /// Right (theirs) range
    pub right_start: usize,
    pub right_end: usize,
    
    /// Type of conflict
    pub conflict_type: ConflictType,
    
    /// Whether this section can be auto-merged
    pub auto_mergeable: bool,
    
    /// Resolved content (if resolved)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_lines: Option<Vec<String>>,
}

impl ThreeWayOpcode {
    pub fn new(
        base_start: usize,
        base_end: usize,
        left_start: usize,
        left_end: usize,
        right_start: usize,
        right_end: usize,
        conflict_type: ConflictType,
        auto_mergeable: bool,
    ) -> Self {
        Self {
            base_start,
            base_end,
            left_start,
            left_end,
            right_start,
            right_end,
            conflict_type,
            auto_mergeable,
            resolved_lines: None,
        }
    }

    pub fn is_conflict(&self) -> bool {
        self.conflict_type != ConflictType::None && !self.auto_mergeable
    }
}

/// Three-way diff result for merge operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreeWayDiffResult {
    pub base_lines: Vec<String>,
    pub left_lines: Vec<String>,
    pub right_lines: Vec<String>,
    pub opcodes: Vec<ThreeWayOpcode>,
    pub conflicts: Vec<usize>, // Indices of conflicting opcodes
    pub merged_lines: Option<Vec<String>>,
}

impl ThreeWayDiffResult {
    pub fn new(
        base_lines: Vec<String>,
        left_lines: Vec<String>,
        right_lines: Vec<String>,
        opcodes: Vec<ThreeWayOpcode>,
    ) -> Self {
        let conflicts: Vec<usize> = opcodes
            .iter()
            .enumerate()
            .filter(|(_, op)| op.is_conflict())
            .map(|(i, _)| i)
            .collect();
        
        Self {
            base_lines,
            left_lines,
            right_lines,
            opcodes,
            conflicts,
            merged_lines: None,
        }
    }

    pub fn has_conflicts(&self) -> bool {
        !self.conflicts.is_empty()
    }

    pub fn conflict_count(&self) -> usize {
        self.conflicts.len()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemType {
    File,
    Folder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemStatus {
    Identical,
    Modified,
    AddedLeft,
    AddedRight,
    DeletedLeft,
    DeletedRight,
    FolderLeftOnly,
    FolderRightOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderItem {
    pub name: String,
    pub item_type: ItemType,
    pub status: ItemStatus,
    pub left_path: Option<String>,
    pub right_path: Option<String>,
}

// Trait implementations for conversions

impl From<&crate::engine::moved_blocks::MovedBlock> for MovedBlockInfo {
    fn from(mb: &crate::engine::moved_blocks::MovedBlock) -> Self {
        Self {
            left_start: mb.left_start,
            left_end: mb.left_end,
            right_start: mb.right_start,
            right_end: mb.right_end,
            size: mb.left_end - mb.left_start,
        }
    }
}

impl From<&crate::engine::filters::DiffOptions> for DiffOptionsInfo {
    fn from(opts: &crate::engine::filters::DiffOptions) -> Self {
        Self {
            ignore_case: opts.ignore_case,
            ignore_whitespace: format!("{:?}", opts.whitespace_mode),
            ignore_blank_lines: opts.ignore_blank_lines,
            ignore_comments: opts.ignore_comments,
        }
    }
}
