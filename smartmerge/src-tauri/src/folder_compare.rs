use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::models::{FolderItem, ItemStatus, ItemType};

pub fn compare_folders(left_path: &str, right_path: &str) -> Result<Vec<FolderItem>, String> {
    let left_dir = Path::new(left_path);
    let right_dir = Path::new(right_path);

    if !left_dir.is_dir() || !right_dir.is_dir() {
        return Ok(Vec::new());
    }

    let left_entries = read_entries(left_dir)?;
    let right_entries = read_entries(right_dir)?;

    let mut all_names = BTreeMap::new();
    for name in left_entries.keys() {
        all_names.insert(name.clone(), ());
    }
    for name in right_entries.keys() {
        all_names.insert(name.clone(), ());
    }

    let mut results = Vec::new();

    for name in all_names.keys() {
        let left = left_entries.get(name);
        let right = right_entries.get(name);

        match (left, right) {
            (Some(left_pathbuf), Some(right_pathbuf)) => {
                let left_is_dir = left_pathbuf.is_dir();
                let right_is_dir = right_pathbuf.is_dir();
                if left_is_dir && right_is_dir {
                    results.push(FolderItem {
                        name: name.clone(),
                        item_type: ItemType::Folder,
                        status: ItemStatus::Identical,
                        left_path: Some(left_pathbuf.to_string_lossy().to_string()),
                        right_path: Some(right_pathbuf.to_string_lossy().to_string()),
                    });
                } else if !left_is_dir && !right_is_dir {
                    let status = compare_files(left_pathbuf, right_pathbuf);
                    results.push(FolderItem {
                        name: name.clone(),
                        item_type: ItemType::File,
                        status,
                        left_path: Some(left_pathbuf.to_string_lossy().to_string()),
                        right_path: Some(right_pathbuf.to_string_lossy().to_string()),
                    });
                } else {
                    results.push(FolderItem {
                        name: name.clone(),
                        item_type: ItemType::File,
                        status: ItemStatus::Modified,
                        left_path: Some(left_pathbuf.to_string_lossy().to_string()),
                        right_path: Some(right_pathbuf.to_string_lossy().to_string()),
                    });
                }
            }
            (Some(left_pathbuf), None) => {
                let item_type = if left_pathbuf.is_dir() {
                    ItemType::Folder
                } else {
                    ItemType::File
                };
                let status = if left_pathbuf.is_dir() {
                    ItemStatus::FolderLeftOnly
                } else {
                    ItemStatus::AddedLeft
                };
                results.push(FolderItem {
                    name: name.clone(),
                    item_type,
                    status,
                    left_path: Some(left_pathbuf.to_string_lossy().to_string()),
                    right_path: None,
                });
            }
            (None, Some(right_pathbuf)) => {
                let item_type = if right_pathbuf.is_dir() {
                    ItemType::Folder
                } else {
                    ItemType::File
                };
                let status = if right_pathbuf.is_dir() {
                    ItemStatus::FolderRightOnly
                } else {
                    ItemStatus::AddedRight
                };
                results.push(FolderItem {
                    name: name.clone(),
                    item_type,
                    status,
                    left_path: None,
                    right_path: Some(right_pathbuf.to_string_lossy().to_string()),
                });
            }
            (None, None) => {}
        }
    }

    Ok(results)
}

pub fn is_folder_navigable(item: &FolderItem) -> bool {
    matches!(item.item_type, ItemType::Folder) && item.left_path.is_some() && item.right_path.is_some()
}

fn read_entries(path: &Path) -> Result<BTreeMap<String, std::path::PathBuf>, String> {
    let mut entries = BTreeMap::new();
    let dir_entries = fs::read_dir(path).map_err(|err| err.to_string())?;
    for entry in dir_entries {
        let entry = entry.map_err(|err| err.to_string())?;
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            entries.insert(name.to_string(), path);
        }
    }
    Ok(entries)
}

fn compare_files(left: &Path, right: &Path) -> ItemStatus {
    let left_content = match fs::read_to_string(left) {
        Ok(content) => content,
        Err(_) => return ItemStatus::Modified,
    };
    let right_content = match fs::read_to_string(right) {
        Ok(content) => content,
        Err(_) => return ItemStatus::Modified,
    };

    if left_content == right_content {
        ItemStatus::Identical
    } else {
        ItemStatus::Modified
    }
}
