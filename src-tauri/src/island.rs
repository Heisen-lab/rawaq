use crate::core_domain::SecurityError;
use crate::security::{CommandGate, PathSanitizer};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

static SHELF_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Represents an item staged in the temporary Quick Shelf.
/// Crucial invariant: A QuickShelfItem is only a memory reference.
/// Adding, removing, or clearing QuickShelfItems NEVER mutates or deletes files on disk.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QuickShelfItem {
    pub id: String,
    pub original_path: String,
    pub file_name: String,
    pub file_size: u64,
    pub is_accessible: bool,
    pub is_directory: bool,
    pub extension: String,
    pub added_at_utc: String,
}

/// Manages the in-memory staging references for the Quick Shelf.
#[derive(Debug, Clone, Default)]
pub struct QuickShelfManager {
    items: Arc<RwLock<Vec<QuickShelfItem>>>,
}

impl QuickShelfManager {
    pub fn new() -> Self {
        Self {
            items: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Add file paths as references to the Quick Shelf.
    /// Validates path security, retrieves metadata if accessible, and stages the reference.
    /// NEVER alters, copies, or removes the source files from disk.
    pub fn add_paths(&self, paths: &[String]) -> Result<Vec<QuickShelfItem>, SecurityError> {
        let mut added = Vec::new();
        let mut items = self.items.write().unwrap();

        for raw_path in paths {
            let validated = PathSanitizer::validate(raw_path, None)?;
            let p = Path::new(&validated.normalized);

            // Check if already in shelf
            if items
                .iter()
                .any(|it| it.original_path == validated.normalized)
            {
                continue;
            }

            let file_name = p
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| validated.normalized.clone());

            let extension = p
                .extension()
                .map(|e| e.to_string_lossy().to_string())
                .unwrap_or_default();

            let (file_size, is_directory, is_accessible) = match fs::metadata(p) {
                Ok(meta) => (meta.len(), meta.is_dir(), true),
                Err(_) => (0, false, false),
            };

            let next_id = format!("shelf-{}", SHELF_ID_COUNTER.fetch_add(1, Ordering::SeqCst));
            let item = QuickShelfItem {
                id: next_id,
                original_path: validated.normalized,
                file_name,
                file_size,
                is_accessible,
                is_directory,
                extension,
                added_at_utc: chrono_now_string(),
            };

            items.push(item.clone());
            added.push(item);
        }

        Ok(added)
    }

    /// List all currently staged references in the Quick Shelf.
    /// Dynamically verifies whether the referenced files still exist on disk.
    pub fn get_items(&self) -> Vec<QuickShelfItem> {
        let mut items = self.items.write().unwrap();
        for item in items.iter_mut() {
            let p = Path::new(&item.original_path);
            if let Ok(meta) = fs::metadata(p) {
                item.is_accessible = true;
                item.file_size = meta.len();
                item.is_directory = meta.is_dir();
            } else {
                item.is_accessible = false;
            }
        }
        items.clone()
    }

    /// Remove a staged item reference by its ID.
    /// The original file on disk is left 100% untouched.
    pub fn remove_item(&self, item_id: &str) -> bool {
        let mut items = self.items.write().unwrap();
        let initial_len = items.len();
        items.retain(|item| item.id != item_id);
        items.len() < initial_len
    }

    /// Clear all staged item references from the shelf.
    /// Invariant: Source files on disk are NEVER modified or deleted.
    pub fn clear(&self) -> usize {
        let mut items = self.items.write().unwrap();
        let count = items.len();
        items.clear();
        count
    }

    /// Returns the number of staged references.
    pub fn count(&self) -> usize {
        self.items.read().unwrap().len()
    }
}

/// The visual visibility state of the Top Capsule / Living Island.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapsuleVisibility {
    Visible,
    Collapsed,
    HiddenFullscreen,
}

/// The reactive state of the Top Capsule.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapsuleState {
    pub visibility: CapsuleVisibility,
    pub previous_visibility: CapsuleVisibility,
    pub is_fullscreen_active: bool,
}

/// Manages fullscreen auto-hide behavior and capsule visibility transitions.
#[derive(Debug, Clone)]
pub struct FullscreenDetector {
    state: Arc<RwLock<CapsuleState>>,
}

impl Default for FullscreenDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl FullscreenDetector {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(CapsuleState {
                visibility: CapsuleVisibility::Visible,
                previous_visibility: CapsuleVisibility::Visible,
                is_fullscreen_active: false,
            })),
        }
    }

    /// Called when an external window enters or exits fullscreen mode.
    /// Automatically hides the capsule when an app enters fullscreen,
    /// and seamlessly restores previous visibility state upon exit.
    pub fn set_fullscreen_active(&self, active: bool) -> CapsuleState {
        let mut state = self.state.write().unwrap();
        if active {
            if state.visibility != CapsuleVisibility::HiddenFullscreen {
                state.previous_visibility = state.visibility;
            }
            state.visibility = CapsuleVisibility::HiddenFullscreen;
            state.is_fullscreen_active = true;
        } else {
            if state.visibility == CapsuleVisibility::HiddenFullscreen {
                state.visibility = state.previous_visibility;
            }
            state.is_fullscreen_active = false;
        }
        state.clone()
    }

    /// Toggles between Expanded (Visible) and Collapsed mode when not in fullscreen.
    pub fn toggle_collapse(&self) -> CapsuleState {
        let mut state = self.state.write().unwrap();
        if state.visibility == CapsuleVisibility::HiddenFullscreen {
            // Do not alter visible state while in fullscreen, but update previous preference
            state.previous_visibility = match state.previous_visibility {
                CapsuleVisibility::Visible => CapsuleVisibility::Collapsed,
                _ => CapsuleVisibility::Visible,
            };
        } else {
            state.visibility = match state.visibility {
                CapsuleVisibility::Visible => CapsuleVisibility::Collapsed,
                CapsuleVisibility::Collapsed => CapsuleVisibility::Visible,
                CapsuleVisibility::HiddenFullscreen => CapsuleVisibility::HiddenFullscreen,
            };
            state.previous_visibility = state.visibility;
        }
        state.clone()
    }

    /// Returns the current capsule state.
    pub fn get_state(&self) -> CapsuleState {
        self.state.read().unwrap().clone()
    }
}

/// The classified category of clipboard content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClipboardContentKind {
    SafeUrl,
    EmailAddress,
    PlainText,
    RejectedDangerous,
}

/// An actionable suggestion derived from inspected clipboard content.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClipboardSuggestion {
    pub kind: ClipboardContentKind,
    pub raw_preview: String,
    pub sanitized_content: String,
    pub title: String,
    pub is_actionable: bool,
    pub rejection_reason: Option<String>,
}

/// Smart Clipboard Opt-In Service:
/// Strictly respects privacy (requires opt_in = true), cleans content,
/// detects content type, and blocks malicious protocol schemes.
pub struct SmartClipboardService;

impl SmartClipboardService {
    /// Process incoming clipboard text with strict security validation.
    /// If `opt_in` is false, returns `Ok(None)` immediately.
    pub fn process_text(
        raw_text: &str,
        opt_in: bool,
    ) -> Result<Option<ClipboardSuggestion>, SecurityError> {
        if !opt_in {
            return Ok(None);
        }

        let trimmed = raw_text.trim();
        if trimmed.is_empty() {
            return Ok(None);
        }

        let lower = trimmed.to_ascii_lowercase();

        // 1. Check for dangerous protocol schemes
        if lower.starts_with("javascript:")
            || lower.starts_with("vbscript:")
            || lower.starts_with("data:")
            || lower.starts_with("file:")
            || lower.contains("javascript:")
            || lower.contains("<script")
        {
            return Ok(Some(ClipboardSuggestion {
                kind: ClipboardContentKind::RejectedDangerous,
                raw_preview: truncate_str(trimmed, 60),
                sanitized_content: String::new(),
                title: "محتوى غير آمن تم حظره".to_string(),
                is_actionable: false,
                rejection_reason: Some("تم حظر المخطط أو النص البرمجي غير المصرح به".to_string()),
            }));
        }

        // 2. Check for Safe Web URL (http:// or https://)
        if lower.starts_with("https://") || lower.starts_with("http://") {
            match CommandGate::sanitize_clipboard_uri(trimmed) {
                Ok(safe_url) => {
                    let title = extract_url_domain_or_title(&safe_url);
                    return Ok(Some(ClipboardSuggestion {
                        kind: ClipboardContentKind::SafeUrl,
                        raw_preview: truncate_str(&safe_url, 60),
                        sanitized_content: safe_url,
                        title,
                        is_actionable: true,
                        rejection_reason: None,
                    }));
                }
                Err(err) => {
                    return Ok(Some(ClipboardSuggestion {
                        kind: ClipboardContentKind::RejectedDangerous,
                        raw_preview: truncate_str(trimmed, 60),
                        sanitized_content: String::new(),
                        title: "رابط غير صالح أو غير آمن".to_string(),
                        is_actionable: false,
                        rejection_reason: Some(format!("فشل تدقيق الرابط: {err:?}")),
                    }));
                }
            }
        }

        // 3. Check for Email Address
        if is_likely_email(trimmed) {
            let sanitized = CommandGate::sanitize_text_payload(trimmed);
            return Ok(Some(ClipboardSuggestion {
                kind: ClipboardContentKind::EmailAddress,
                raw_preview: truncate_str(&sanitized, 60),
                sanitized_content: sanitized.clone(),
                title: format!("بريد إلكتروني: {sanitized}"),
                is_actionable: true,
                rejection_reason: None,
            }));
        }

        // 4. Default to Plain Text (sanitized against XSS/HTML injections)
        let sanitized = CommandGate::sanitize_text_payload(trimmed);
        if sanitized.is_empty() {
            return Ok(None);
        }

        let title = truncate_str(&sanitized, 40);
        Ok(Some(ClipboardSuggestion {
            kind: ClipboardContentKind::PlainText,
            raw_preview: truncate_str(trimmed, 60),
            sanitized_content: sanitized,
            title,
            is_actionable: true,
            rejection_reason: None,
        }))
    }
}

fn truncate_str(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_chars).collect();
        format!("{truncated}...")
    }
}

fn extract_url_domain_or_title(url: &str) -> String {
    let without_scheme = url
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    let domain = without_scheme.split('/').next().unwrap_or(without_scheme);
    format!("رابط: {domain}")
}

fn is_likely_email(s: &str) -> bool {
    let s = s.trim();
    if s.contains('@') && s.contains('.') && !s.contains(' ') && !s.contains('/') {
        let parts: Vec<&str> = s.split('@').collect();
        if parts.len() == 2 && !parts[0].is_empty() && parts[1].contains('.') {
            return true;
        }
    }
    false
}

fn chrono_now_string() -> String {
    let now = std::time::SystemTime::now();
    let duration = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}.{:03}Z", duration.as_secs(), duration.subsec_millis())
}

// ─── Automated Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    /// Test 1: `island::test_quick_shelf_holds_references_without_moving_source`
    /// Adding files to the temporary shelf and emptying it without deleting or modifying
    /// original files on disk.
    #[test]
    fn test_quick_shelf_holds_references_without_moving_source() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let dir_path = temp_dir.path();

        let file1_path = dir_path.join("تقرير_المشروع_النهائي.docx");
        let file2_path = dir_path.join("screenshot_2026.png");
        let file3_path = dir_path.join("بيانات_العملاء.csv");

        let content1 = b"Word Document Project Report Content 2026";
        let content2 = b"PNG Image Binary Data 123456789";
        let content3 = b"CSV Customer Data, 100, Verified";

        File::create(&file1_path)
            .unwrap()
            .write_all(content1)
            .unwrap();
        File::create(&file2_path)
            .unwrap()
            .write_all(content2)
            .unwrap();
        File::create(&file3_path)
            .unwrap()
            .write_all(content3)
            .unwrap();

        let shelf = QuickShelfManager::new();

        // 1. Add files to shelf
        let paths = vec![
            file1_path.to_string_lossy().to_string(),
            file2_path.to_string_lossy().to_string(),
            file3_path.to_string_lossy().to_string(),
        ];

        let added = shelf.add_paths(&paths).expect("Should add paths to shelf");
        assert_eq!(added.len(), 3);
        assert_eq!(shelf.count(), 3);

        // Verify items in shelf
        let items = shelf.get_items();
        assert_eq!(items.len(), 3);
        assert!(items.iter().all(|it| it.is_accessible));

        // CRUCIAL: All files must still exist on disk with untouched contents
        assert!(file1_path.exists());
        assert!(file2_path.exists());
        assert!(file3_path.exists());
        assert_eq!(fs::read(&file1_path).unwrap(), content1);
        assert_eq!(fs::read(&file2_path).unwrap(), content2);
        assert_eq!(fs::read(&file3_path).unwrap(), content3);

        // 2. Remove a single reference from shelf
        let first_id = items[0].id.clone();
        let removed = shelf.remove_item(&first_id);
        assert!(removed);
        assert_eq!(shelf.count(), 2);

        // File 1 MUST still exist on disk!
        assert!(
            file1_path.exists(),
            "Original file 1 must NOT be deleted from disk on shelf remove"
        );
        assert_eq!(fs::read(&file1_path).unwrap(), content1);

        // 3. Clear the whole shelf
        let cleared_count = shelf.clear();
        assert_eq!(cleared_count, 2);
        assert_eq!(shelf.count(), 0);

        // ALL files MUST still exist on disk!
        assert!(
            file1_path.exists(),
            "Original file 1 must NOT be deleted from disk on shelf clear"
        );
        assert!(
            file2_path.exists(),
            "Original file 2 must NOT be deleted from disk on shelf clear"
        );
        assert!(
            file3_path.exists(),
            "Original file 3 must NOT be deleted from disk on shelf clear"
        );

        // 4. Test external file deletion gracefully flagged as inaccessible
        let paths_again = vec![file1_path.to_string_lossy().to_string()];
        shelf.add_paths(&paths_again).unwrap();
        assert!(shelf.get_items()[0].is_accessible);

        // Remove the file on disk externally
        fs::remove_file(&file1_path).unwrap();
        let items_after_external_delete = shelf.get_items();
        assert_eq!(items_after_external_delete.len(), 1);
        assert!(
            !items_after_external_delete[0].is_accessible,
            "Shelf must gracefully report missing disk file as is_accessible = false"
        );
    }

    /// Test 2: `island::test_fullscreen_detection_hides_and_restores_capsule`
    /// Simulate app entering fullscreen mode and verify capsule state transitions
    /// to hidden, then returns seamlessly when exiting fullscreen.
    #[test]
    fn test_fullscreen_detection_hides_and_restores_capsule() {
        let detector = FullscreenDetector::new();
        let initial_state = detector.get_state();
        assert_eq!(initial_state.visibility, CapsuleVisibility::Visible);
        assert!(!initial_state.is_fullscreen_active);

        // 1. User toggles to collapsed mode
        let collapsed = detector.toggle_collapse();
        assert_eq!(collapsed.visibility, CapsuleVisibility::Collapsed);

        // 2. Fullscreen app opens (e.g. video player or game)
        let state_fs = detector.set_fullscreen_active(true);
        assert_eq!(
            state_fs.visibility,
            CapsuleVisibility::HiddenFullscreen,
            "Capsule must hide when fullscreen is active"
        );
        assert!(state_fs.is_fullscreen_active);
        assert_eq!(state_fs.previous_visibility, CapsuleVisibility::Collapsed);

        // While in fullscreen, capsule remains hidden
        let current = detector.get_state();
        assert_eq!(current.visibility, CapsuleVisibility::HiddenFullscreen);

        // 3. Fullscreen app exits
        let state_restored = detector.set_fullscreen_active(false);
        assert_eq!(
            state_restored.visibility,
            CapsuleVisibility::Collapsed,
            "Capsule must restore to previous collapsed state when exiting fullscreen"
        );
        assert!(!state_restored.is_fullscreen_active);

        // 4. Test starting from Visible state and entering fullscreen
        let visible_state = detector.toggle_collapse();
        assert_eq!(visible_state.visibility, CapsuleVisibility::Visible);

        let state_fs2 = detector.set_fullscreen_active(true);
        assert_eq!(state_fs2.visibility, CapsuleVisibility::HiddenFullscreen);
        assert_eq!(state_fs2.previous_visibility, CapsuleVisibility::Visible);

        let state_restored2 = detector.set_fullscreen_active(false);
        assert_eq!(
            state_restored2.visibility,
            CapsuleVisibility::Visible,
            "Capsule must restore to Visible state when exiting fullscreen"
        );
    }

    /// Test 3: `island::test_clipboard_sanitizer_blocks_malicious_uris`
    /// Reject dangerous protocols like `javascript:` or unauthorized schemes when capturing links.
    #[test]
    fn test_clipboard_sanitizer_blocks_malicious_uris() {
        // Opt-out test: When user has not opted-in, returns None
        let res_no_opt_in = SmartClipboardService::process_text("https://example.com", false)
            .expect("Should succeed");
        assert!(
            res_no_opt_in.is_none(),
            "Must not process clipboard when opt_in is false"
        );

        // 1. Malicious: javascript: alert payload
        let xss_payload = "javascript:alert(document.cookie)";
        let res_js = SmartClipboardService::process_text(xss_payload, true).unwrap();
        assert!(res_js.is_some());
        let suggestion_js = res_js.unwrap();
        assert_eq!(
            suggestion_js.kind,
            ClipboardContentKind::RejectedDangerous,
            "javascript: scheme must be rejected as dangerous"
        );
        assert!(!suggestion_js.is_actionable);

        // 2. Malicious: data: html payload
        let data_payload = "data:text/html;base64,PHNjcmlwdD5hbGVydCgxKTwvc2NyaXB0Pg==";
        let res_data = SmartClipboardService::process_text(data_payload, true).unwrap();
        assert_eq!(
            res_data.unwrap().kind,
            ClipboardContentKind::RejectedDangerous,
            "data: scheme must be rejected as dangerous"
        );

        // 3. Malicious: vbscript payload
        let vbscript_payload = "vbscript:MsgBox('Injected')";
        let res_vb = SmartClipboardService::process_text(vbscript_payload, true).unwrap();
        assert_eq!(
            res_vb.unwrap().kind,
            ClipboardContentKind::RejectedDangerous,
            "vbscript: scheme must be rejected as dangerous"
        );

        // 4. Malicious: file:// scheme
        let file_payload = "file:///C:/Windows/System32/cmd.exe";
        let res_file = SmartClipboardService::process_text(file_payload, true).unwrap();
        assert_eq!(
            res_file.unwrap().kind,
            ClipboardContentKind::RejectedDangerous,
            "file: scheme must be rejected as dangerous"
        );

        // 5. Valid safe HTTPS URL
        let safe_url = "https://rawaq.app/guide/stage-5?ref=desktop";
        let res_url = SmartClipboardService::process_text(safe_url, true).unwrap();
        assert!(res_url.is_some());
        let suggestion_url = res_url.unwrap();
        assert_eq!(suggestion_url.kind, ClipboardContentKind::SafeUrl);
        assert_eq!(suggestion_url.sanitized_content, safe_url);
        assert!(suggestion_url.is_actionable);

        // 6. Valid email
        let email = "support@rawaq.app";
        let res_email = SmartClipboardService::process_text(email, true).unwrap();
        let suggestion_email = res_email.unwrap();
        assert_eq!(suggestion_email.kind, ClipboardContentKind::EmailAddress);
        assert_eq!(suggestion_email.sanitized_content, email);

        // 7. Plain text with HTML tags stripped to clean PlainText
        let html_note = "<b>اجتماع الفريق</b> في تمام الساعة 10 صباحًا";
        let res_note = SmartClipboardService::process_text(html_note, true).unwrap();
        let suggestion_note = res_note.unwrap();
        assert_eq!(suggestion_note.kind, ClipboardContentKind::PlainText);
        assert!(!suggestion_note.sanitized_content.contains("<b>"));
        assert!(suggestion_note.sanitized_content.contains("اجتماع الفريق"));

        // 8. Script tag payload rejected as dangerous
        let script_payload = "<script>alert('XSS')</script>";
        let res_script = SmartClipboardService::process_text(script_payload, true).unwrap();
        assert_eq!(
            res_script.unwrap().kind,
            ClipboardContentKind::RejectedDangerous,
            "Script tag must be rejected as dangerous"
        );
    }
}
