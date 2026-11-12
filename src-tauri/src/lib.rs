pub mod boxes;
pub mod core_domain;
pub mod display;
pub mod i18n;
pub mod island;
pub mod organizer;
pub mod productivity;
pub mod release;
pub mod search;
pub mod security;
pub mod storage;

use crate::boxes::{BoxService, FolderState};
use crate::display::{
    BoxPlacement, DisplayInfo, DisplayRect, DisplayTopologyManager, ResourceSaverReport,
    TopologyChangeReport,
};
use crate::i18n::{verify_embedded_catalogs, ParityReport};
use crate::island::{
    CapsuleState, ClipboardSuggestion, FullscreenDetector, QuickShelfItem, QuickShelfManager,
    SmartClipboardService,
};
use crate::organizer::{DryRunPlan, ExecutionReport, SafeOrganizerEngine, UndoReport};
use crate::productivity::{CaptureManager, PomodoroStatus, PomodoroTimer, TaskManager};
use crate::release::{ReleaseManager, ReleaseVerificationReport};
use crate::search::{SearchResponse, SearchableFile, UnifiedSearchEngine};
use crate::security::{CommandGate, PathSanitizer, ValidatedPath};
use crate::storage::{CaptureNote, DesktopBox, OrganizeRule, Storage, TaskItem};
use std::path::Path;
use std::sync::{OnceLock, RwLock};

static GLOBAL_SHELF: OnceLock<QuickShelfManager> = OnceLock::new();
static GLOBAL_DETECTOR: OnceLock<FullscreenDetector> = OnceLock::new();
static GLOBAL_POMODORO: OnceLock<PomodoroTimer> = OnceLock::new();
static GLOBAL_TOPOLOGY: OnceLock<RwLock<DisplayTopologyManager>> = OnceLock::new();

fn get_shelf() -> &'static QuickShelfManager {
    GLOBAL_SHELF.get_or_init(QuickShelfManager::new)
}

fn get_detector() -> &'static FullscreenDetector {
    GLOBAL_DETECTOR.get_or_init(FullscreenDetector::new)
}

fn get_pomodoro() -> &'static PomodoroTimer {
    GLOBAL_POMODORO.get_or_init(PomodoroTimer::new)
}

fn get_topology() -> &'static RwLock<DisplayTopologyManager> {
    GLOBAL_TOPOLOGY.get_or_init(|| RwLock::new(DisplayTopologyManager::new()))
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

#[tauri::command]
fn create_task_command(
    db_path: String,
    title: String,
    description: Option<String>,
    period: Option<String>,
    tags: Option<String>,
) -> Result<TaskItem, String> {
    let storage = Storage::open(std::path::Path::new(&db_path)).map_err(|e| e.to_string())?;
    TaskManager::create_task(
        &storage,
        &title,
        description.as_deref(),
        period.as_deref(),
        tags.as_deref(),
    )
}

#[tauri::command]
fn list_tasks_command(db_path: String, period: Option<String>) -> Result<Vec<TaskItem>, String> {
    let storage = Storage::open(std::path::Path::new(&db_path)).map_err(|e| e.to_string())?;
    TaskManager::list_tasks(&storage, period.as_deref())
}

#[tauri::command]
fn toggle_task_command(db_path: String, id: i64) -> Result<bool, String> {
    let storage = Storage::open(std::path::Path::new(&db_path)).map_err(|e| e.to_string())?;
    TaskManager::toggle_task(&storage, id)
}

#[tauri::command]
fn update_task_command(
    db_path: String,
    id: i64,
    title: String,
    description: Option<String>,
    period: Option<String>,
    tags: Option<String>,
) -> Result<TaskItem, String> {
    let storage = Storage::open(std::path::Path::new(&db_path)).map_err(|e| e.to_string())?;
    TaskManager::update_task(
        &storage,
        id,
        &title,
        description.as_deref(),
        period.as_deref(),
        tags.as_deref(),
    )
}

#[tauri::command]
fn delete_task_command(db_path: String, id: i64) -> Result<(), String> {
    let storage = Storage::open(std::path::Path::new(&db_path)).map_err(|e| e.to_string())?;
    TaskManager::delete_task(&storage, id)
}

#[tauri::command]
fn create_capture_note_command(
    db_path: String,
    content: String,
    note_type: Option<String>,
    tags: Option<String>,
    source_url: Option<String>,
) -> Result<CaptureNote, String> {
    let storage = Storage::open(std::path::Path::new(&db_path)).map_err(|e| e.to_string())?;
    CaptureManager::create_capture(
        &storage,
        &content,
        note_type.as_deref(),
        tags.as_deref(),
        source_url.as_deref(),
    )
}

#[tauri::command]
fn list_capture_notes_command(
    db_path: String,
    limit: Option<usize>,
) -> Result<Vec<CaptureNote>, String> {
    let storage = Storage::open(std::path::Path::new(&db_path)).map_err(|e| e.to_string())?;
    CaptureManager::list_captures(&storage, limit)
}

#[tauri::command]
fn delete_capture_note_command(db_path: String, id: i64) -> Result<(), String> {
    let storage = Storage::open(std::path::Path::new(&db_path)).map_err(|e| e.to_string())?;
    CaptureManager::delete_capture(&storage, id)
}

#[tauri::command]
fn pomodoro_start_command(task_id: Option<i64>, duration_seconds: Option<u64>) -> PomodoroStatus {
    get_pomodoro().start(task_id, duration_seconds)
}

#[tauri::command]
fn pomodoro_pause_command() -> PomodoroStatus {
    get_pomodoro().pause()
}

#[tauri::command]
fn pomodoro_resume_command() -> PomodoroStatus {
    get_pomodoro().resume()
}

#[tauri::command]
fn pomodoro_tick_command(delta_seconds: u64) -> PomodoroStatus {
    get_pomodoro().tick(delta_seconds)
}

#[tauri::command]
fn pomodoro_stop_command() -> PomodoroStatus {
    get_pomodoro().stop()
}

#[tauri::command]
fn pomodoro_status_command() -> PomodoroStatus {
    get_pomodoro().get_status()
}

#[tauri::command]
fn pomodoro_save_command(db_path: String) -> Result<(), String> {
    let storage = Storage::open(std::path::Path::new(&db_path)).map_err(|e| e.to_string())?;
    get_pomodoro()
        .save_to_storage(&storage)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn pomodoro_restore_command(db_path: String) -> Result<PomodoroStatus, String> {
    let storage = Storage::open(std::path::Path::new(&db_path)).map_err(|e| e.to_string())?;
    get_pomodoro()
        .load_from_storage(&storage)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn unified_search_command(
    db_path: String,
    query: String,
    files: Option<Vec<SearchableFile>>,
) -> Result<SearchResponse, String> {
    let storage = Storage::open(std::path::Path::new(&db_path)).map_err(|e| e.to_string())?;
    let file_list = files.unwrap_or_default();
    Ok(UnifiedSearchEngine::search(&storage, &query, &file_list))
}

#[tauri::command]
fn register_display_command(display: DisplayInfo) -> Result<TopologyChangeReport, String> {
    let mut topology = get_topology().write().map_err(|e| e.to_string())?;
    Ok(topology.register_display(display))
}

#[tauri::command]
fn unregister_display_command(display_id: String) -> Result<TopologyChangeReport, String> {
    let mut topology = get_topology().write().map_err(|e| e.to_string())?;
    topology
        .unregister_display(&display_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn list_displays_command() -> Result<Vec<DisplayInfo>, String> {
    let topology = get_topology().read().map_err(|e| e.to_string())?;
    Ok(topology.list_displays())
}

#[tauri::command]
fn place_box_command(
    box_id: u64,
    target_display_id: String,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> Result<DisplayRect, String> {
    let mut topology = get_topology().write().map_err(|e| e.to_string())?;
    topology
        .place_box(box_id, &target_display_id, x, y, width, height)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_box_placement_command(box_id: u64) -> Result<Option<BoxPlacement>, String> {
    let topology = get_topology().read().map_err(|e| e.to_string())?;
    Ok(topology.get_placement(box_id).cloned())
}

#[tauri::command]
fn list_box_placements_command() -> Result<Vec<BoxPlacement>, String> {
    let topology = get_topology().read().map_err(|e| e.to_string())?;
    Ok(topology.list_placements())
}

#[tauri::command]
fn set_display_resolution_command(
    display_id: String,
    new_bounds: DisplayRect,
    new_scale: f64,
) -> Result<TopologyChangeReport, String> {
    let mut topology = get_topology().write().map_err(|e| e.to_string())?;
    topology
        .set_display_resolution(&display_id, new_bounds, new_scale)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn toggle_resource_saver_command(enabled: bool) -> Result<ResourceSaverReport, String> {
    let mut topology = get_topology().write().map_err(|e| e.to_string())?;
    Ok(topology.toggle_resource_saver(enabled))
}

#[tauri::command]
fn set_box_collapsed_command(box_id: u64, collapsed: bool) -> Result<bool, String> {
    let mut topology = get_topology().write().map_err(|e| e.to_string())?;
    topology
        .set_box_collapsed(box_id, collapsed)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn generate_release_checksums_command(release_dir: String) -> Result<String, String> {
    let (sums, _) = ReleaseManager::generate_sha256sums_from_dir(Path::new(&release_dir))
        .map_err(|e| e.to_string())?;
    Ok(sums)
}

#[tauri::command]
fn verify_release_checksums_command(
    manifest_content: String,
    release_dir: String,
) -> Result<ReleaseVerificationReport, String> {
    ReleaseManager::verify_sha256sums(&manifest_content, Path::new(&release_dir))
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
            save_clipboard_note_command,
            create_task_command,
            list_tasks_command,
            toggle_task_command,
            update_task_command,
            delete_task_command,
            create_capture_note_command,
            list_capture_notes_command,
            delete_capture_note_command,
            pomodoro_start_command,
            pomodoro_pause_command,
            pomodoro_resume_command,
            pomodoro_tick_command,
            pomodoro_stop_command,
            pomodoro_status_command,
            pomodoro_save_command,
            pomodoro_restore_command,
            unified_search_command,
            register_display_command,
            unregister_display_command,
            list_displays_command,
            place_box_command,
            get_box_placement_command,
            list_box_placements_command,
            set_display_resolution_command,
            toggle_resource_saver_command,
            set_box_collapsed_command,
            generate_release_checksums_command,
            verify_release_checksums_command
        ])
        .run(tauri::generate_context!())
        .expect("error while running Rawaq Tauri application");
}
