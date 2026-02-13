mod engine;
mod file_io;
mod folder_compare;
mod highlighting;
mod line_mapping;
mod merge;
mod models;
mod trivial_filter;

use engine::{DiffEngine, AlgorithmType};
use file_io::{read_lines, write_lines};
use folder_compare::compare_folders;
use highlighting::{HighlightEngine, Theme};
use line_mapping::{compute_line_mappings, GhostLineLayout};
use merge::MergeEngine;
use models::{DiffResult, FolderItem, Opcode};
use trivial_filter::{analyze_trivial_changes, filter_trivial_opcodes, get_trivial_opcodes, TrivialChangeStats};
use tauri::image::Image;
use tauri::menu::{IconMenuItem, CheckMenuItem, MenuBuilder, SubmenuBuilder};
use tauri::{Emitter, Manager, Wry};

struct MenuItems {
    back: IconMenuItem<Wry>,
    save: IconMenuItem<Wry>,
    save_all: IconMenuItem<Wry>,
    undo: IconMenuItem<Wry>,
    redo: IconMenuItem<Wry>,
    prev: IconMenuItem<Wry>,
    next: IconMenuItem<Wry>,
    copy_left: IconMenuItem<Wry>,
    copy_right: IconMenuItem<Wry>,
    copy_all_left: IconMenuItem<Wry>,
    copy_all_right: IconMenuItem<Wry>,
    toggle_minimap_locations: IconMenuItem<Wry>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct MenuStatePayload {
    can_back: bool,
    can_save: bool,
    can_save_all: bool,
    can_undo: bool,
    can_redo: bool,
    can_navigate: bool,
    can_copy: bool,
    can_copy_all: bool,
}

#[tauri::command]
fn set_menu_state(state: tauri::State<MenuItems>, payload: MenuStatePayload) -> Result<(), String> {
    state
        .back
        .set_enabled(payload.can_back)
        .map_err(|e| e.to_string())?;
    state
        .save
        .set_enabled(payload.can_save)
        .map_err(|e| e.to_string())?;
    state
        .save_all
        .set_enabled(payload.can_save_all)
        .map_err(|e| e.to_string())?;
    state
        .undo
        .set_enabled(payload.can_undo)
        .map_err(|e| e.to_string())?;
    state
        .redo
        .set_enabled(payload.can_redo)
        .map_err(|e| e.to_string())?;
    state
        .prev
        .set_enabled(payload.can_navigate)
        .map_err(|e| e.to_string())?;
    state
        .next
        .set_enabled(payload.can_navigate)
        .map_err(|e| e.to_string())?;
    state
        .copy_left
        .set_enabled(payload.can_copy)
        .map_err(|e| e.to_string())?;
    state
        .copy_right
        .set_enabled(payload.can_copy)
        .map_err(|e| e.to_string())?;
    state
        .copy_all_left
        .set_enabled(payload.can_copy_all)
        .map_err(|e| e.to_string())?;
    state
        .copy_all_right
        .set_enabled(payload.can_copy_all)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn compare_files(left_path: String, right_path: String, engine: String) -> Result<DiffResult, String> {
    let left_lines = read_lines(&left_path)?;
    let right_lines = read_lines(&right_path)?;

    let algorithm_type = match engine.as_str() {
        "myers" => AlgorithmType::Myers,
        "smart" => AlgorithmType::Smart,
        _ => AlgorithmType::Myers,
    };

    let diff_engine = DiffEngine::with_algorithm(algorithm_type);
    let result = diff_engine.compute_diff_result(left_lines, right_lines);

    Ok(result)
}

#[tauri::command]
fn compare_lines(left_lines: Vec<String>, right_lines: Vec<String>, engine: String) -> Result<Vec<Opcode>, String> {
    let algorithm_type = match engine.as_str() {
        "myers" => AlgorithmType::Myers,
        "smart" => AlgorithmType::Smart,
        _ => AlgorithmType::Myers,
    };

    let diff_engine = DiffEngine::with_algorithm(algorithm_type);
    let opcodes = diff_engine.compute_diff(&left_lines, &right_lines);

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

#[tauri::command]
fn get_path_kind(path: String) -> Result<String, String> {
    let metadata = std::fs::metadata(&path).map_err(|_| "missing".to_string())?;
    if metadata.is_file() {
        Ok("file".to_string())
    } else if metadata.is_dir() {
        Ok("dir".to_string())
    } else {
        Ok("other".to_string())
    }
}

#[tauri::command]
fn highlight_diff(
    left_lines: Vec<String>,
    right_lines: Vec<String>,
    algorithm: String,
    theme: String,
) -> Result<highlighting::HighlightResult, String> {
    // Select algorithm
    let algorithm_type = match algorithm.as_str() {
        "myers" => AlgorithmType::Myers,
        "smart" => AlgorithmType::Smart,
        "patience" => AlgorithmType::Patience,
        _ => AlgorithmType::Myers,
    };

    // Select theme
    let theme_type = match theme.as_str() {
        "light" => Theme::Light,
        "dark" => Theme::Dark,
        _ => Theme::Auto,
    };

    // Compute diff
    let diff_engine = DiffEngine::with_algorithm(algorithm_type);
    let diff_result = diff_engine.compute_diff_result(left_lines.clone(), right_lines.clone());

    // Generate highlights
    let highlight_engine = HighlightEngine::new(theme_type);
    let highlight_result = highlight_engine.highlight(&diff_result, &left_lines, &right_lines);

    Ok(highlight_result)
}

#[tauri::command]
fn highlight_three_way(
    base_lines: Vec<String>,
    left_lines: Vec<String>,
    right_lines: Vec<String>,
    theme: String,
) -> Result<highlighting::ThreeWayHighlightResult, String> {
    use engine::three_way::ThreeWayDiff;

    // Select theme
    let theme_type = match theme.as_str() {
        "light" => Theme::Light,
        "dark" => Theme::Dark,
        _ => Theme::Auto,
    };

    // Compute three-way diff
    let diff_engine = DiffEngine::with_algorithm(AlgorithmType::Myers);
    let three_way_engine = ThreeWayDiff::new(diff_engine);
    let merge_result = three_way_engine.diff(&base_lines, &left_lines, &right_lines);

    // Generate highlights
    let highlight_engine = HighlightEngine::new(theme_type);
    let highlight_result = highlight_engine.highlight_three_way(&merge_result, &base_lines, &left_lines, &right_lines);

    Ok(highlight_result)
}

#[tauri::command]
fn get_available_color_schemes() -> Result<Vec<(String, String)>, String> {
    Ok(highlighting::ColorScheme::available_schemes())
}

#[tauri::command]
fn get_color_scheme(scheme_id: String) -> Result<highlighting::ColorScheme, String> {
    highlighting::ColorScheme::get_scheme(&scheme_id)
        .ok_or_else(|| format!("Color scheme not found: {}", scheme_id))
}

#[tauri::command]
fn analyze_diff_for_trivial(
    mut diff_result: DiffResult,
    treat_case_as_trivial: bool,
) -> Result<(DiffResult, TrivialChangeStats), String> {
    let stats = analyze_trivial_changes(&mut diff_result, treat_case_as_trivial);
    Ok((diff_result, stats))
}

#[tauri::command]
fn get_trivial_stats(
    diff_result: DiffResult,
    treat_case_as_trivial: bool,
) -> Result<TrivialChangeStats, String> {
    let mut result = diff_result;
    let stats = analyze_trivial_changes(&mut result, treat_case_as_trivial);
    Ok(stats)
}

#[tauri::command]
fn filter_out_trivial_changes(opcodes: Vec<Opcode>) -> Result<Vec<Opcode>, String> {
    Ok(filter_trivial_opcodes(&opcodes))
}

#[tauri::command]
fn get_only_trivial_changes(opcodes: Vec<Opcode>) -> Result<Vec<Opcode>, String> {
    Ok(get_trivial_opcodes(&opcodes))
}

#[tauri::command]
fn compute_ghost_line_mappings(
    left_lines: Vec<String>,
    right_lines: Vec<String>,
    opcodes: Vec<Opcode>,
) -> Result<Vec<GhostLineLayout>, String> {
    Ok(compute_line_mappings(&left_lines, &right_lines, &opcodes))
}

use std::sync::Mutex;
use std::collections::HashMap;
use once_cell::sync::Lazy;

// Global merge engine storage (thread-safe)
static MERGE_ENGINES: Lazy<Mutex<HashMap<String, MergeEngine>>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[tauri::command]
fn start_merge(
    merge_id: String,
    base_lines: Vec<String>,
    left_lines: Vec<String>,
    right_lines: Vec<String>,
) -> Result<merge::MergeState, String> {
    use engine::three_way::ThreeWayDiff;

    // Compute three-way diff
    let diff_engine = DiffEngine::with_algorithm(AlgorithmType::Myers);
    let three_way_engine = ThreeWayDiff::new(diff_engine);
    let diff_result = three_way_engine.diff(&base_lines, &left_lines, &right_lines);

    // Create merge engine
    let engine = MergeEngine::new(diff_result, base_lines, left_lines, right_lines);
    let state = engine.state().clone();

    // Store engine
    let mut engines = MERGE_ENGINES.lock().unwrap();
    engines.insert(merge_id, engine);

    Ok(state)
}

#[tauri::command]
fn merge_accept_left(merge_id: String, block_index: usize) -> Result<merge::MergeState, String> {
    let mut engines = MERGE_ENGINES.lock().unwrap();
    let engine = engines.get_mut(&merge_id).ok_or("Merge session not found")?;
    engine.accept_left(block_index)?;
    Ok(engine.state().clone())
}

#[tauri::command]
fn merge_accept_right(merge_id: String, block_index: usize) -> Result<merge::MergeState, String> {
    let mut engines = MERGE_ENGINES.lock().unwrap();
    let engine = engines.get_mut(&merge_id).ok_or("Merge session not found")?;
    engine.accept_right(block_index)?;
    Ok(engine.state().clone())
}

#[tauri::command]
fn merge_accept_both(merge_id: String, block_index: usize, left_first: bool) -> Result<merge::MergeState, String> {
    let mut engines = MERGE_ENGINES.lock().unwrap();
    let engine = engines.get_mut(&merge_id).ok_or("Merge session not found")?;
    engine.accept_both(block_index, left_first)?;
    Ok(engine.state().clone())
}

#[tauri::command]
fn merge_apply_custom(merge_id: String, block_index: usize, custom_lines: Vec<String>) -> Result<merge::MergeState, String> {
    let mut engines = MERGE_ENGINES.lock().unwrap();
    let engine = engines.get_mut(&merge_id).ok_or("Merge session not found")?;
    engine.apply_custom(block_index, custom_lines)?;
    Ok(engine.state().clone())
}

#[tauri::command]
fn merge_undo(merge_id: String) -> Result<merge::MergeState, String> {
    let mut engines = MERGE_ENGINES.lock().unwrap();
    let engine = engines.get_mut(&merge_id).ok_or("Merge session not found")?;
    engine.undo()?;
    Ok(engine.state().clone())
}

#[tauri::command]
fn merge_redo(merge_id: String) -> Result<merge::MergeState, String> {
    let mut engines = MERGE_ENGINES.lock().unwrap();
    let engine = engines.get_mut(&merge_id).ok_or("Merge session not found")?;
    engine.redo()?;
    Ok(engine.state().clone())
}

#[tauri::command]
fn merge_auto_resolve(merge_id: String) -> Result<(usize, merge::MergeState), String> {
    let mut engines = MERGE_ENGINES.lock().unwrap();
    let engine = engines.get_mut(&merge_id).ok_or("Merge session not found")?;
    let count = engine.auto_resolve_all()?;
    Ok((count, engine.state().clone()))
}

#[tauri::command]
fn merge_build_result(merge_id: String, include_unresolved: bool) -> Result<merge::MergeResult, String> {
    let engines = MERGE_ENGINES.lock().unwrap();
    let engine = engines.get(&merge_id).ok_or("Merge session not found")?;
    
    if include_unresolved {
        engine.build_result()
    } else {
        engine.build_partial_result()
    }
}

fn load_icon(bytes: &'static [u8]) -> tauri::Result<Image<'static>> {
    Image::from_bytes(bytes)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let icon_open_files = load_icon(include_bytes!("../icons/menu/open-files.png"))?;
            let icon_open_folders = load_icon(include_bytes!("../icons/menu/open-folders.png"))?;
            let icon_back = load_icon(include_bytes!("../icons/menu/back.png"))?;
            let icon_save = load_icon(include_bytes!("../icons/menu/save.png"))?;
            let icon_save_all = load_icon(include_bytes!("../icons/menu/save-all.png"))?;
            let icon_undo = load_icon(include_bytes!("../icons/menu/undo.png"))?;
            let icon_redo = load_icon(include_bytes!("../icons/menu/redo.png"))?;
            let icon_prev = load_icon(include_bytes!("../icons/menu/prev.png"))?;
            let icon_next = load_icon(include_bytes!("../icons/menu/next.png"))?;
            let icon_copy_left = load_icon(include_bytes!("../icons/menu/copy-left.png"))?;
            let icon_copy_right = load_icon(include_bytes!("../icons/menu/copy-right.png"))?;
            let icon_copy_all_left = load_icon(include_bytes!("../icons/menu/copy-all-left.png"))?;
            let icon_copy_all_right = load_icon(include_bytes!("../icons/menu/copy-all-right.png"))?;
            let icon_theme = load_icon(include_bytes!("../icons/menu/theme.png"))?;
            let icon_about = load_icon(include_bytes!("../icons/menu/about.png"))?;
            let icon_quit = load_icon(include_bytes!("../icons/menu/quit.png"))?;
            let icon_font = load_icon(include_bytes!("../icons/menu/font.png"))?;
            let open_files = IconMenuItem::with_id(
                app,
                "open-files",
                "Open Files...",
                true,
                Some(icon_open_files.clone()),
                Some("CmdOrCtrl+O"),
            )?;
            let open_folders = IconMenuItem::with_id(
                app,
                "open-folders",
                "Open Folders...",
                true,
                Some(icon_open_folders.clone()),
                Some("CmdOrCtrl+Shift+O"),
            )?;
            let save = IconMenuItem::with_id(
                app,
                "save",
                "Save",
                true,
                Some(icon_save.clone()),
                Some("CmdOrCtrl+S"),
            )?;
            let save_all = IconMenuItem::with_id(
                app,
                "save-all",
                "Save All",
                true,
                Some(icon_save_all.clone()),
                Some("CmdOrCtrl+Shift+S"),
            )?;
            let quit = IconMenuItem::with_id(
                app,
                "quit",
                "Quit",
                true,
                Some(icon_quit.clone()),
                Some("CmdOrCtrl+Q"),
            )?;
            let undo = IconMenuItem::with_id(
                app,
                "undo",
                "Undo",
                true,
                Some(icon_undo.clone()),
                Some("CmdOrCtrl+Z"),
            )?;
            let redo = IconMenuItem::with_id(
                app,
                "redo",
                "Redo",
                true,
                Some(icon_redo.clone()),
                Some("CmdOrCtrl+Shift+Z"),
            )?;
            let font_picker = IconMenuItem::with_id(
                app,
                "font-picker",
                "Font...",
                true,
                Some(icon_font.clone()),
                None::<&str>,
            )?;
            let back = IconMenuItem::with_id(
                app,
                "back",
                "Back to Folders",
                true,
                Some(icon_back.clone()),
                None::<&str>,
            )?;
            let prev = IconMenuItem::with_id(
                app,
                "prev",
                "Previous Change",
                true,
                Some(icon_prev.clone()),
                Some("Alt+Up"),
            )?;
            let next = IconMenuItem::with_id(
                app,
                "next",
                "Next Change",
                true,
                Some(icon_next.clone()),
                Some("Alt+Down"),
            )?;
            let copy_left = IconMenuItem::with_id(
                app,
                "copy-left",
                "Copy to Left",
                true,
                Some(icon_copy_left.clone()),
                Some("Alt+Left"),
            )?;
            let copy_right = IconMenuItem::with_id(
                app,
                "copy-right",
                "Copy to Right",
                true,
                Some(icon_copy_right.clone()),
                Some("Alt+Right"),
            )?;
            let copy_all_left = IconMenuItem::with_id(
                app,
                "copy-all-left",
                "Copy All to Left",
                true,
                Some(icon_copy_all_left.clone()),
                Some("CmdOrCtrl+Alt+Left"),
            )?;
            let copy_all_right = IconMenuItem::with_id(
                app,
                "copy-all-right",
                "Copy All to Right",
                true,
                Some(icon_copy_all_right.clone()),
                Some("CmdOrCtrl+Alt+Right"),
            )?;
            let theme = IconMenuItem::with_id(
                app,
                "theme",
                "Toggle Theme",
                true,
                Some(icon_theme.clone()),
                None::<&str>,
            )?;

            let icon_minimap = load_icon(include_bytes!("../icons/menu/minimap.png"))?;
            let toggle_minimap_locations = IconMenuItem::with_id(
                app,
                "toggle-minimap-locations",
                "Toggle Minimap & Locations",
                true,
                Some(icon_minimap.clone()),
                None::<&str>,
            )?;
            let about = IconMenuItem::with_id(
                app,
                "about",
                "About SmartMerge",
                true,
                Some(icon_about.clone()),
                None::<&str>,
            )?;

            back.set_enabled(false)?;
            save.set_enabled(false)?;
            save_all.set_enabled(false)?;
            undo.set_enabled(false)?;
            redo.set_enabled(false)?;
            prev.set_enabled(false)?;
            next.set_enabled(false)?;
            copy_left.set_enabled(false)?;
            copy_right.set_enabled(false)?;
            copy_all_left.set_enabled(false)?;
            copy_all_right.set_enabled(false)?;

            let file_menu = SubmenuBuilder::new(app, "&File")
                .item(&open_files)
                .item(&open_folders)
                .separator()
                .item(&save)
                .item(&save_all)
                .separator()
                .item(&quit)
                .build()?;

            let edit_menu = SubmenuBuilder::new(app, "&Edit")
                .item(&undo)
                .item(&redo)
                .separator()
                .item(&font_picker)
                .build()?;

            let navigate_menu = SubmenuBuilder::new(app, "&Navigate")
                .item(&back)
                .item(&prev)
                .item(&next)
                .build()?;

            let merge_menu = SubmenuBuilder::new(app, "&Merge")
                .item(&copy_left)
                .item(&copy_right)
                .separator()
                .item(&copy_all_left)
                .item(&copy_all_right)
                .build()?;

            let view_menu = SubmenuBuilder::new(app, "&View")
                .item(&toggle_minimap_locations)
                .item(&theme)
                .build()?;

            let help_menu = SubmenuBuilder::new(app, "&Help")
                .item(&about)
                .build()?;

            let menu = MenuBuilder::new(app)
                .item(&file_menu)
                .item(&edit_menu)
                .item(&navigate_menu)
                .item(&merge_menu)
                .item(&view_menu)
                .item(&help_menu)
                .build()?;
            app.set_menu(menu)?;

            app.manage(MenuItems {
                back,
                save,
                save_all,
                undo,
                redo,
                prev,
                next,
                copy_left,
                copy_right,
                copy_all_left,
                copy_all_right,
                toggle_minimap_locations,
            });

            Ok(())
        })
        .on_menu_event(|app, event| {
            let id = event.id().as_ref();
            if id == "quit" {
                app.exit(0);
                return;
            }
            let _ = app.emit("menu-action", id.to_string());
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            compare_files,
            compare_lines,
            compare_folders_command,
            save_file,
            set_menu_state,
            get_path_kind,
            highlight_diff,
            highlight_three_way,
            get_available_color_schemes,
            get_color_scheme,
            analyze_diff_for_trivial,
            get_trivial_stats,
            filter_out_trivial_changes,
            get_only_trivial_changes,
            compute_ghost_line_mappings,
            start_merge,
            merge_accept_left,
            merge_accept_right,
            merge_accept_both,
            merge_apply_custom,
            merge_undo,
            merge_redo,
            merge_auto_resolve,
            merge_build_result
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
