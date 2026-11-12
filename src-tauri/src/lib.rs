pub mod boxes;
pub mod core_domain;
pub mod i18n;
pub mod island;
pub mod organizer;
pub mod security;
pub mod storage;

use crate::boxes::{BoxService, FolderState};
use crate::i18n::{verify_embedded_catalogs, ParityReport};
use crate::island::{
    CapsuleState, ClipboardSuggestion, FullscreenDetector, QuickShelfItem, QuickShelfManager,
    SmartClipboardService,
};
use crate::organizer::{DryRunPlan, ExecutionReport, SafeOrganizerEngine, UndoReport};
use crate::security::{CommandGate, PathSanitizer, ValidatedPath};
use crate::storage::{CaptureNote, DesktopBox, OrganizeRule, Storage};
use std::sync::OnceLock;

static GLOBAL_SHELF: OnceLock<QuickShelfManager> = OnceLock::new();
static GLOBAL_DETECTOR: OnceLock<FullscreenDetector> = OnceLock::new();

fn get_shelf() -> &'static QuickShelfManager {
    GLOBAL_SHELF.get_or_init(QuickShelfManager::new)
}

fn get_detector() -> &'static FullscreenDetector {
    GLOBAL_DETECTOR.get_or_init(FullscreenDetector::new)
}

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

#[tauri::command]
fn preview_organization_command(
    source_folder: String,
    rules: Vec<OrganizeRule>,
) -> Result<DryRunPlan, String> {
    SafeOrganizerEngine::compute_dry_run(std::path::Path::new(&source_folder), &rules)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn execute_organization_command(
    db_path: String,
    plan: DryRunPlan,
) -> Result<ExecutionReport, String> {
    let storage = Storage::open(std::path::Path::new(&db_path)).map_err(|e| e.to_string())?;
    SafeOrganizerEngine::execute_plan(&storage, &plan, None).map_err(|e| e.to_string())
}

#[tauri::command]
fn undo_organization_command(db_path: String, batch_id: String) -> Result<UndoReport, String> {
    let storage = Storage::open(std::path::Path::new(&db_path)).map_err(|e| e.to_string())?;
    SafeOrganizerEngine::undo_batch(&storage, &batch_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_undo_batches_command(db_path: String) -> Result<Vec<String>, String> {
    let storage = Storage::open(std::path::Path::new(&db_path)).map_err(|e| e.to_string())?;
    storage.list_undo_batches().map_err(|e| e.to_string())
}

#[tauri::command]
fn add_to_quick_shelf_command(paths: Vec<String>) -> Result<Vec<QuickShelfItem>, String> {
    get_shelf().add_paths(&paths).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_quick_shelf_items_command() -> Vec<QuickShelfItem> {
    get_shelf().get_items()
}

#[tauri::command]
fn remove_from_quick_shelf_command(item_id: String) -> bool {
    get_shelf().remove_item(&item_id)
}

#[tauri::command]
fn clear_quick_shelf_command() -> usize {
    get_shelf().clear()
}

#[tauri::command]
fn set_fullscreen_active_command(active: bool) -> CapsuleState {
    get_detector().set_fullscreen_active(active)
}

#[tauri::command]
fn toggle_capsule_collapse_command() -> CapsuleState {
    get_detector().toggle_collapse()
}

#[tauri::command]
fn get_capsule_state_command() -> CapsuleState {
    get_detector().get_state()
}

#[tauri::command]
fn process_clipboard_command(
    raw_text: String,
    opt_in: bool,
) -> Result<Option<ClipboardSuggestion>, String> {
    SmartClipboardService::process_text(&raw_text, opt_in).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_clipboard_note_command(
    db_path: String,
    content: String,
    source_url: Option<String>,
    note_type: Option<String>,
) -> Result<i64, String> {
    let storage = Storage::open(std::path::Path::new(&db_path)).map_err(|e| e.to_string())?;
    let sanitized_content = CommandGate::sanitize_text_payload(&content);
    let note = CaptureNote {
        id: None,
        content: sanitized_content,
        note_type: note_type.unwrap_or_else(|| "quick_capture".to_string()),
        tags: "[\"island\", \"clipboard\"]".to_string(),
        source_url,
        created_at: None,
    };
    storage
        .insert_capture_note(&note)
        .map_err(|e| e.to_string())
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
            dissolve_group_command,
            preview_organization_command,
            execute_organization_command,
            undo_organization_command,
            list_undo_batches_command,
            add_to_quick_shelf_command,
            get_quick_shelf_items_command,
            remove_from_quick_shelf_command,
            clear_quick_shelf_command,
            set_fullscreen_active_command,
            toggle_capsule_collapse_command,
            get_capsule_state_command,
            process_clipboard_command,
            save_clipboard_note_command
        ])
        .run(tauri::generate_context!())
        .expect("error while running Rawaq Tauri application");
}
