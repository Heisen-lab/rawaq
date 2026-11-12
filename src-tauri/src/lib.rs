pub mod core_domain;
pub mod i18n;
pub mod security;
pub mod storage;

use crate::i18n::{verify_embedded_catalogs, ParityReport};
use crate::security::{CommandGate, PathSanitizer, ValidatedPath};

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            validate_path_command,
            sanitize_text_command,
            check_i18n_parity_command
        ])
        .run(tauri::generate_context!())
        .expect("error while running Rawaq Tauri application");
}
