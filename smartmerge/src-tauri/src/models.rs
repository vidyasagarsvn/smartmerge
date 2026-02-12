use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Opcode {
    pub tag: String,
    pub i1: usize,
    pub i2: usize,
    pub j1: usize,
    pub j2: usize,
}

impl Opcode {
    pub fn new(tag: &str, i1: usize, i2: usize, j1: usize, j2: usize) -> Self {
        Self {
            tag: tag.to_string(),
            i1,
            i2,
            j1,
            j2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResult {
    pub left_lines: Vec<String>,
    pub right_lines: Vec<String>,
    pub opcodes: Vec<Opcode>,
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
