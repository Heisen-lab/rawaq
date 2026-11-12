pub mod boxes;
pub mod core_domain;
pub mod i18n;
pub mod security;
pub mod storage;

use crate::boxes::{BoxService, FolderState};
use crate::i18n::{verify_embedded_catalogs, ParityReport};
use crate::security::{CommandGate, PathSanitizer, ValidatedPath};
use crate::storage::{DesktopBox, Storage};

#[tauri::command]
fn validate_path_command(
    raw_path: String,
    allowed_roots: Option<Vec<String>>,
) -> Result<ValidatedPath, String> {
    PathSanitizer::validate(&raw_path, allowed_roots.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
fn sanitize_text_command(raw_text: String) -> String {
    CommandGate::sanitize_text_payload(&raw_text)
}

#[tauri::command]
fn check_i18n_parity_command() -> Result<ParityReport, String> {
    verify_embedded_catalogs()
}

#[tauri::command]
fn scan_box_contents_command(folder_path: String, sort_order: Option<String>) -> FolderState {
    BoxService::scan_folder_contents(
        &folder_path,
        &sort_order.unwrap_or_else(|| "name".to_string()),
    )
}

#[tauri::command]
fn create_box_command(
    db_path: String,
    name: String,
    folder_path: String,
    display_mode: Option<String>,
    sort_order: Option<String>,
) -> Result<DesktopBox, String> {
    let storage = Storage::open(std::path::Path::new(&db_path)).map_err(|e| e.to_string())?;
    BoxService::create_box(&storage, name, folder_path, display_mode, sort_order)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn update_box_command(db_path: String, b: DesktopBox) -> Result<(), String> {
    let storage = Storage::open(std::path::Path::new(&db_path)).map_err(|e| e.to_string())?;
    BoxService::update_box(&storage, &b).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_box_command(db_path: String, box_id: i64) -> Result<(), String> {
    let storage = Storage::open(std::path::Path::new(&db_path)).map_err(|e| e.to_string())?;
    BoxService::delete_box(&storage, box_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn merge_boxes_command(
    db_path: String,
    group_name: String,
    box_ids: Vec<i64>,
) -> Result<i64, String> {
    let storage = Storage::open(std::path::Path::new(&db_path)).map_err(|e| e.to_string())?;
    BoxService::merge_boxes_into_group(&storage, &group_name, &box_ids).map_err(|e| e.to_string())
}

#[tauri::command]
fn dissolve_group_command(db_path: String, group_id: i64) -> Result<(), String> {
    let storage = Storage::open(std::path::Path::new(&db_path)).map_err(|e| e.to_string())?;
    BoxService::dissolve_group(&storage, group_id).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            validate_path_command,
            sanitize_text_command,
            check_i18n_parity_command,
            scan_box_contents_command,
            create_box_command,
            update_box_command,
            delete_box_command,
            merge_boxes_command,
            dissolve_group_command
        ])
        .run(tauri::generate_context!())
        .expect("error while running Rawaq Tauri application");
}
