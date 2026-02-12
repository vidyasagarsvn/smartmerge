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
use tauri::image::Image;
use tauri::menu::{IconMenuItem, MenuBuilder, SubmenuBuilder};
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
}

#[derive(serde::Deserialize)]
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

fn color_icon(rgb: [u8; 3]) -> Image<'static> {
    let size = 16u32;
    let mut rgba = vec![0u8; (size * size * 4) as usize];
    let center = (size as f32 - 1.0) / 2.0;
    let radius = 7.0f32;
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            let idx = ((y * size + x) * 4) as usize;
            if dx * dx + dy * dy <= radius * radius {
                rgba[idx] = rgb[0];
                rgba[idx + 1] = rgb[1];
                rgba[idx + 2] = rgb[2];
                rgba[idx + 3] = 255;
            }
        }
    }
    Image::new_owned(rgba, size, size)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let icon_open_files = color_icon([75, 123, 236]);
            let icon_open_folders = color_icon([244, 184, 74]);
            let icon_back = color_icon([108, 92, 231]);
            let icon_save = color_icon([39, 174, 96]);
            let icon_save_all = color_icon([26, 188, 156]);
            let icon_undo = color_icon([242, 153, 74]);
            let icon_redo = color_icon([242, 153, 74]);
            let icon_prev = color_icon([155, 89, 182]);
            let icon_next = color_icon([155, 89, 182]);
            let icon_copy_left = color_icon([231, 111, 81]);
            let icon_copy_right = color_icon([231, 111, 81]);
            let icon_copy_all_left = color_icon([231, 111, 81]);
            let icon_copy_all_right = color_icon([231, 111, 81]);
            let icon_engine = color_icon([0, 184, 148]);
            let icon_theme = color_icon([255, 209, 102]);
            let icon_about = color_icon([108, 117, 125]);
            let icon_quit = color_icon([231, 111, 81]);
            let icon_font = color_icon([75, 123, 236]);
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
            let engine = IconMenuItem::with_id(
                app,
                "engine",
                "Toggle Engine",
                true,
                Some(icon_engine.clone()),
                None::<&str>,
            )?;
            let theme = IconMenuItem::with_id(
                app,
                "theme",
                "Toggle Theme",
                true,
                Some(icon_theme.clone()),
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
                .item(&engine)
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
            set_menu_state
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
