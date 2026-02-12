mod diff_engine;
mod file_io;
mod folder_compare;
mod models;
mod smart_diff;

use diff_engine::{myers_opcodes, normalize_opcodes};
use file_io::{read_lines, write_lines};
use folder_compare::compare_folders;
use models::{DiffResult, FolderItem, Opcode};
use smart_diff::smart_diff;

#[tauri::command]
fn compare_files(left_path: String, right_path: String, engine: String) -> Result<DiffResult, String> {
    let left_lines = read_lines(&left_path)?;
    let right_lines = read_lines(&right_path)?;

    let opcodes = if engine == "myers" {
        let raw = myers_opcodes(&left_lines, &right_lines);
        normalize_opcodes(&raw, left_lines.len(), right_lines.len())
    } else {
        smart_diff(&left_lines, &right_lines)
    };

    Ok(DiffResult {
        left_lines,
        right_lines,
        opcodes,
    })
}

#[tauri::command]
fn compare_lines(left_lines: Vec<String>, right_lines: Vec<String>, engine: String) -> Result<Vec<Opcode>, String> {
    let opcodes = if engine == "myers" {
        let raw = myers_opcodes(&left_lines, &right_lines);
        normalize_opcodes(&raw, left_lines.len(), right_lines.len())
    } else {
        smart_diff(&left_lines, &right_lines)
    };

    Ok(opcodes)
}

#[tauri::command]
fn compare_folders_command(left_path: String, right_path: String) -> Result<Vec<FolderItem>, String> {
    compare_folders(&left_path, &right_path)
}

#[tauri::command]
fn save_file(path: String, lines: Vec<String>) -> Result<(), String> {
    write_lines(&path, &lines)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            compare_files,
            compare_lines,
            compare_folders_command,
            save_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
